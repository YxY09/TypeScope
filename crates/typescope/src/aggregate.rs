//! 物理按键的内存聚合。热路径只改计数，不分配，不写盘。

pub const IDLE_MS_DEFAULT: u64 = 5_000;
const RING: usize = 4096;

#[derive(Clone, Copy, Debug, Default)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

impl Mods {
    pub fn bits(self) -> u8 {
        (self.shift as u8) | ((self.ctrl as u8) << 1) | ((self.alt as u8) << 2) | ((self.win as u8) << 3)
    }

    pub fn any(self) -> bool {
        self.bits() != 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KeySample {
    pub vk: u16,
    pub down: bool,
    pub repeat: bool,
    pub injected: bool,
    pub at_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeakRow {
    pub best_10: u32,
    pub best_30: u32,
    pub best_60: u32,
    pub sessions: u32,
    pub longest_ms: u64,
    pub first_minute: i32,
    pub last_minute: i32,
}

impl Default for PeakRow {
    fn default() -> Self {
        Self {
            best_10: 0,
            best_30: 0,
            best_60: 0,
            sessions: 0,
            longest_ms: 0,
            first_minute: -1,
            last_minute: -1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HourBucket {
    pub physical: u32,
    pub repeats: u32,
    pub injected: u32,
    pub backspace: u32,
    pub delete_key: u32,
    pub space: u32,
    pub enter: u32,
    pub shift: u32,
    pub ctrl: u32,
    pub alt: u32,
    pub active_ms: u64,
}

#[derive(Clone, Debug)]
pub struct Engine {
    pub idle_ms: u64,
    down: [bool; 256],
    keys: Vec<(u32, [u32; 256])>,
    hours: Vec<((u32, u8), HourBucket)>,
    intervals: Vec<(u32, [u32; 12])>,
    shortcuts: Vec<(u32, u8, u16, u32)>,
    ring: [u64; RING],
    ring_i: usize,
    ring_n: usize,
    last_physical_ms: Option<u64>,
    session_start: Option<u64>,
    session_keys: u32,
    session_active: u64,
    pub sessions: u32,
    pub longest_session_ms: u64,
    pub best_10: u32,
    pub best_30: u32,
    pub best_60: u32,
    pub paused: bool,
    pub app_stats: bool,
    session_day: u32,
    peaks: Vec<(u32, PeakRow)>,
    pending: bool,
    pub last_counted_gap: u64,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            idle_ms: IDLE_MS_DEFAULT,
            down: [false; 256],
            keys: Vec::new(),
            hours: Vec::new(),
            intervals: Vec::new(),
            shortcuts: Vec::new(),
            ring: [0; RING],
            ring_i: 0,
            ring_n: 0,
            last_physical_ms: None,
            session_start: None,
            session_keys: 0,
            session_active: 0,
            sessions: 0,
            longest_session_ms: 0,
            best_10: 0,
            best_30: 0,
            best_60: 0,
            paused: false,
            app_stats: false,
            session_day: 0,
            peaks: Vec::new(),
            pending: false,
            last_counted_gap: 0,
        }
    }
}

impl Engine {
    pub fn clear_downs(&mut self) {
        self.down = [false; 256];
    }

    /// 返回真表示这次是一次新的物理按下，已经计入按键数。
    pub fn observe(&mut self, day: u32, hour: u8, sample: KeySample, mods: Mods) -> bool {
        self.last_counted_gap = 0;
        if self.paused || sample.vk > 255 {
            return false;
        }
        let vk = sample.vk as usize;
        if !sample.down {
            self.down[vk] = false;
            return false;
        }
        if sample.injected {
            self.hour(day, hour).injected += 1;
            self.pending = true;
            return false;
        }
        if sample.repeat || self.down[vk] {
            self.down[vk] = true;
            self.hour(day, hour).repeats += 1;
            self.pending = true;
            return false;
        }
        self.down[vk] = true;
        {
            let bucket = self.hour(day, hour);
            bucket.physical += 1;
            match sample.vk {
                0x08 => bucket.backspace += 1,
                0x2E => bucket.delete_key += 1,
                0x20 => bucket.space += 1,
                0x0D => bucket.enter += 1,
                0x10 | 0xA0 | 0xA1 => bucket.shift += 1,
                0x11 | 0xA2 | 0xA3 => bucket.ctrl += 1,
                0x12 | 0xA4 | 0xA5 => bucket.alt += 1,
                _ => {}
            }
        }
        self.key_row(day)[vk] += 1;
        if mods.any() && !is_modifier(sample.vk) {
            self.add_shortcut(day, mods.bits(), sample.vk);
        }
        self.note_time(day, hour, sample.at_ms);
        self.pending = true;
        true
    }

    pub fn stamp_clock(&mut self, day: u32, minute: u16) {
        let row = self.peak_row(day);
        let minute = minute as i32;
        if row.first_minute < 0 {
            row.first_minute = minute;
        }
        row.last_minute = minute;
    }

    pub fn suspend(&mut self) {
        self.close_open_session();
        self.last_physical_ms = None;
        self.clear_downs();
    }

    pub fn has_pending(&self) -> bool {
        self.pending
    }

    pub fn mark_clean(&mut self) {
        self.pending = false;
    }

    pub fn copy_peaks(&self) -> Vec<(u32, PeakRow)> {
        self.peaks.clone()
    }

    /// 只清掉已经成功写入数据库的会话次数。速度纪录留在内存里，数据库用最大值合并。
    pub fn clear_flushed_sessions(&mut self) {
        for (_, row) in &mut self.peaks {
            row.sessions = 0;
        }
        self.sessions = 0;
    }

    /// 清理已落盘的旧日期峰值记录，只保留 `keep_day` 当天并清零已落盘的 `sessions`，防止跨天常驻无限增长。
    pub fn prune_flushed_peaks(&mut self, keep_day: u32) {
        self.peaks.retain(|(d, _)| *d == keep_day);
        for (_, row) in &mut self.peaks {
            row.sessions = 0;
        }
        self.sessions = 0;
    }

    pub fn untake_hours(&mut self, rows: Vec<((u32, u8), HourBucket)>) {
        for ((day, hour), src) in rows {
            let dst = self.hour(day, hour);
            dst.physical += src.physical;
            dst.repeats += src.repeats;
            dst.injected += src.injected;
            dst.backspace += src.backspace;
            dst.delete_key += src.delete_key;
            dst.space += src.space;
            dst.enter += src.enter;
            dst.shift += src.shift;
            dst.ctrl += src.ctrl;
            dst.alt += src.alt;
            dst.active_ms += src.active_ms;
        }
    }

    pub fn untake_keys(&mut self, rows: Vec<(u32, [u32; 256])>) {
        for (day, counts) in rows {
            let dst = self.key_row(day);
            for (i, n) in counts.iter().enumerate() {
                dst[i] += *n;
            }
        }
    }

    pub fn untake_intervals(&mut self, rows: Vec<(u32, [u32; 12])>) {
        for (day, buckets) in rows {
            let dst = self.interval_row(day);
            for (i, n) in buckets.iter().enumerate() {
                dst[i] += *n;
            }
        }
    }

    pub fn untake_shortcuts(&mut self, rows: Vec<(u32, u8, u16, u32)>) {
        for (day, mods, vk, n) in rows {
            if n == 0 {
                continue;
            }
            self.add_shortcut(day, mods, vk);
            if let Some(row) = self.shortcuts.iter_mut().find(|(d, m, v, _)| *d == day && *m == mods && *v == vk) {
                row.3 += n.saturating_sub(1);
            }
        }
    }

    fn note_time(&mut self, day: u32, hour: u8, at_ms: u64) {
        if let Some(prev) = self.last_physical_ms {
            let gap = at_ms.saturating_sub(prev);
            self.interval_row(day)[interval_bucket(gap)] += 1;
            if gap <= self.idle_ms {
                self.hour(day, hour).active_ms += gap;
                self.session_active += gap;
                self.last_counted_gap = gap;
            } else {
                self.close_session(prev);
            }
        }
        if self.session_start.is_none() {
            self.session_start = Some(at_ms);
            self.session_keys = 0;
            self.session_active = 0;
            self.session_day = day;
            self.sessions += 1;
            self.peak_row(day).sessions += 1;
        }
        self.session_keys += 1;
        self.last_physical_ms = Some(at_ms);
        self.push_ring(at_ms);
        let best_10 = self.rolling(at_ms, 10_000, 20).unwrap_or(0);
        let best_30 = self.rolling(at_ms, 30_000, 40).unwrap_or(0);
        let best_60 = self.rolling(at_ms, 60_000, 60).unwrap_or(0);
        self.best_10 = self.best_10.max(best_10);
        self.best_30 = self.best_30.max(best_30);
        self.best_60 = self.best_60.max(best_60);
        let row = self.peak_row(day);
        row.best_10 = row.best_10.max(best_10);
        row.best_30 = row.best_30.max(best_30);
        row.best_60 = row.best_60.max(best_60);
    }

    fn rolling(&self, now_ms: u64, window_ms: u64, min_keys: u32) -> Option<u32> {
        if window_ms == 0 || self.ring_n == 0 {
            return None;
        }
        let start = now_ms.saturating_sub(window_ms);
        let mut n = 0u32;
        let origin = if self.ring_n == RING { self.ring_i } else { 0 };
        for i in 0..self.ring_n {
            let t = self.ring[(origin + i) % RING];
            if t > start && t <= now_ms {
                n += 1;
            }
        }
        if n < min_keys {
            return None;
        }
        Some((n as u64 * 60_000 / window_ms) as u32)
    }

    pub fn close_open_session(&mut self) {
        if let Some(prev) = self.last_physical_ms {
            self.close_session(prev);
        }
    }

    fn close_session(&mut self, end_ms: u64) {
        if let Some(start) = self.session_start.take() {
            let dur = end_ms.saturating_sub(start).max(self.session_active);
            self.longest_session_ms = self.longest_session_ms.max(dur);
            let day = self.session_day;
            let row = self.peak_row(day);
            row.longest_ms = row.longest_ms.max(dur);
        }
        self.session_keys = 0;
        self.session_active = 0;
    }

    fn push_ring(&mut self, at_ms: u64) {
        self.ring[self.ring_i] = at_ms;
        self.ring_i = (self.ring_i + 1) % RING;
        self.ring_n = (self.ring_n + 1).min(RING);
    }

    pub fn live_kpm(&self, now_ms: u64, window_ms: u64, min_keys: u32) -> Option<u32> {
        self.rolling(now_ms, window_ms, min_keys)
    }

    pub fn take_hours(&mut self) -> Vec<((u32, u8), HourBucket)> {
        std::mem::take(&mut self.hours)
    }

    pub fn take_keys(&mut self) -> Vec<(u32, [u32; 256])> {
        std::mem::take(&mut self.keys)
    }

    pub fn take_intervals(&mut self) -> Vec<(u32, [u32; 12])> {
        std::mem::take(&mut self.intervals)
    }

    pub fn take_shortcuts(&mut self) -> Vec<(u32, u8, u16, u32)> {
        std::mem::take(&mut self.shortcuts)
    }

    fn hour(&mut self, day: u32, hour: u8) -> &mut HourBucket {
        if let Some(i) = self.hours.iter().position(|((d, h), _)| *d == day && *h == hour) {
            return &mut self.hours[i].1;
        }
        self.hours.push(((day, hour), HourBucket::default()));
        &mut self.hours.last_mut().unwrap().1
    }

    fn key_row(&mut self, day: u32) -> &mut [u32; 256] {
        if let Some(i) = self.keys.iter().position(|(d, _)| *d == day) {
            return &mut self.keys[i].1;
        }
        self.keys.push((day, [0; 256]));
        &mut self.keys.last_mut().unwrap().1
    }

    fn interval_row(&mut self, day: u32) -> &mut [u32; 12] {
        if let Some(i) = self.intervals.iter().position(|(d, _)| *d == day) {
            return &mut self.intervals[i].1;
        }
        self.intervals.push((day, [0; 12]));
        &mut self.intervals.last_mut().unwrap().1
    }

    fn peak_row(&mut self, day: u32) -> &mut PeakRow {
        if let Some(i) = self.peaks.iter().position(|(d, _)| *d == day) {
            return &mut self.peaks[i].1;
        }
        self.peaks.push((day, PeakRow::default()));
        &mut self.peaks.last_mut().unwrap().1
    }

    fn add_shortcut(&mut self, day: u32, mods: u8, vk: u16) {
        if let Some((_, _, _, n)) = self.shortcuts.iter_mut().find(|(d, m, v, _)| *d == day && *m == mods && *v == vk) {
            *n += 1;
            return;
        }
        self.shortcuts.push((day, mods, vk, 1));
    }
}

pub fn is_modifier(vk: u16) -> bool {
    matches!(vk, 0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5)
}

fn interval_bucket(gap_ms: u64) -> usize {
    const EDGES: [u64; 11] = [30, 50, 80, 120, 200, 300, 500, 800, 1200, 2000, 5000];
    EDGES.iter().position(|e| gap_ms < *e).unwrap_or(11)
}

/// 速度 = 窗口内的物理按下次数 / 窗口长度。
/// 分母固定是窗口全长，不用“最后一次减第一次”把窗口缩小。
/// 次数不够 `min_keys` 时返回空，避免一两下按键变成虚高纪录。
pub fn rolling_kpm(stamps: &[u64], now_ms: u64, window_ms: u64, min_keys: u32) -> Option<u32> {
    if window_ms == 0 {
        return None;
    }
    let start = now_ms.saturating_sub(window_ms);
    let n = stamps.iter().filter(|t| **t > start && **t <= now_ms).count() as u32;
    if n < min_keys {
        return None;
    }
    Some((n as u64 * 60_000 / window_ms) as u32)
}

pub fn average_kpm(physical: u32, active_ms: u64) -> Option<u32> {
    if physical == 0 || active_ms == 0 {
        return None;
    }
    Some((physical as u64 * 60_000 / active_ms) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn down(vk: u16, at: u64) -> KeySample {
        KeySample { vk, down: true, repeat: false, injected: false, at_ms: at }
    }

    #[test]
    fn repeat_and_injected_are_not_physical() {
        let mut e = Engine::default();
        e.observe(20261031, 10, down(0x41, 0), Mods::default());
        e.observe(20261031, 10, KeySample { vk: 0x41, down: true, repeat: true, injected: false, at_ms: 30 }, Mods::default());
        e.observe(20261031, 10, KeySample { vk: 0x42, down: true, repeat: false, injected: true, at_ms: 40 }, Mods::default());
        let hours = e.take_hours();
        assert_eq!(hours[0].1.physical, 1);
        assert_eq!(hours[0].1.repeats, 1);
        assert_eq!(hours[0].1.injected, 1);
    }

    #[test]
    fn idle_gap_is_outside_active_time() {
        let mut e = Engine::default();
        e.observe(20261031, 1, down(0x41, 0), Mods::default());
        e.observe(20261031, 1, down(0x42, 100), Mods::default());
        e.observe(20261031, 1, down(0x43, 200), Mods::default());
        e.observe(20261031, 1, down(0x44, 10_000), Mods::default());
        let hours = e.take_hours();
        assert_eq!(hours[0].1.physical, 4);
        assert_eq!(hours[0].1.active_ms, 200);
        assert_eq!(average_kpm(4, 200), Some(1200));
    }

    #[test]
    fn tiny_burst_does_not_set_a_record() {
        let mut e = Engine::default();
        e.observe(1, 0, down(0x41, 0), Mods::default());
        e.observe(1, 0, down(0x42, 50), Mods::default());
        assert_eq!(e.best_10, 0);
        assert!(e.live_kpm(50, 10_000, 5).is_none());
    }

    #[test]
    fn full_window_is_the_speed_denominator() {
        let mut stamps = Vec::new();
        for i in 1..=20 {
            stamps.push(i * 500);
        }
        assert_eq!(rolling_kpm(&stamps, 10_000, 10_000, 20), Some(120));
        assert!(rolling_kpm(&stamps[..2], 10_000, 10_000, 20).is_none());
    }

    #[test]
    fn session_count_can_be_cleared_after_flush() {
        let mut e = Engine::default();
        e.observe(20261031, 1, down(0x41, 0), Mods::default());
        e.observe(20261031, 1, down(0x42, 100), Mods::default());
        e.suspend();
        let peaks = e.copy_peaks();
        assert_eq!(peaks[0].1.sessions, 1);
        e.clear_flushed_sessions();
        assert_eq!(e.copy_peaks()[0].1.sessions, 0);
        assert_eq!(e.sessions, 0);
    }

    #[test]
    fn held_key_without_repeat_flag_counts_once() {
        let mut e = Engine::default();
        let s = down(0x41, 0);
        e.observe(1, 0, s, Mods::default());
        e.observe(1, 0, KeySample { at_ms: 40, ..s }, Mods::default());
        e.observe(1, 0, KeySample { down: false, at_ms: 80, ..s }, Mods::default());
        e.observe(1, 0, KeySample { at_ms: 120, ..s }, Mods::default());
        assert_eq!(e.take_hours()[0].1.physical, 2);
    }

    #[test]
    fn prune_flushed_peaks_removes_old_days_and_keeps_today() {
        let mut e = Engine::default();
        e.observe(20261030, 9, down(0x41, 0), Mods::default());
        e.stamp_clock(20261030, 540);
        e.observe(20261031, 10, down(0x42, 6_000), Mods::default());
        e.stamp_clock(20261031, 600);
        assert_eq!(e.copy_peaks().len(), 2);
        e.prune_flushed_peaks(20261031);
        let peaks = e.copy_peaks();
        assert_eq!(peaks.len(), 1);
        assert_eq!(peaks[0].0, 20261031);
        assert_eq!(peaks[0].1.sessions, 0);
        assert_eq!(peaks[0].1.first_minute, 600);
    }
}
