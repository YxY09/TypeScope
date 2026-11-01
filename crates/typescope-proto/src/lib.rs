//! 小狼毫提交文本的进程间传递，以及把一段已提交文本分成字符类别。
//!
//! 环里的内容只在内存中短暂存在。采集进程读完会把槽位清掉。
//! 这里没有按键序列，也没有磁盘日志。

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub const MAP_NAME: &str = "Local\\TypeScopeRime";
pub const EVENT_NAME: &str = "Local\\TypeScopeRimeEvent";
pub const INIT_MUTEX: &str = "Local\\TypeScopeRimeInit";
pub const CONTROL_EVENT: &str = "Local\\TypeScopeControl";
pub const AGENT_MUTEX: &str = "Local\\TypeScopeAgent";
pub const APP_MUTEX: &str = "Local\\TypeScopeApp";

pub const MAGIC: u32 = 0x5453_4331;
pub const VERSION: u32 = 1;
pub const SLOTS: u32 = 64;
pub const SLOT_BYTES: usize = 768;
const HEADER: usize = 64;
const SLOT_STRIDE: usize = 8 + SLOT_BYTES;

pub const fn map_len() -> usize {
    HEADER + SLOTS as usize * SLOT_STRIDE
}

const OFF_MAGIC: usize = 0;
const OFF_VERSION: usize = 4;
const OFF_HEAD: usize = 8;
const OFF_TAIL: usize = 12;
const OFF_ACCEPTED: usize = 16;
const OFF_DROPPED: usize = 24;
const OFF_HELLO: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClassCounts {
    pub han: u32,
    pub latin: u32,
    pub digit: u32,
    pub punct: u32,
    pub space: u32,
    pub other: u32,
}

impl ClassCounts {
    pub fn chars(self) -> u32 {
        self.han + self.latin + self.digit + self.punct + self.space + self.other
    }
}

pub fn classify(text: &str) -> ClassCounts {
    let mut out = ClassCounts::default();
    for ch in text.chars() {
        if is_han(ch) {
            out.han += 1;
        } else if ch.is_ascii_alphabetic() {
            out.latin += 1;
        } else if ch.is_ascii_digit() {
            out.digit += 1;
        } else if ch.is_whitespace() {
            out.space += 1;
        } else if is_punct(ch) {
            out.punct += 1;
        } else {
            out.other += 1;
        }
    }
    out
}

fn is_han(ch: char) -> bool {
    matches!(ch as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x2A6DF | 0x2A700..=0x2B73F
    )
}

fn is_punct(ch: char) -> bool {
    if ch.is_ascii_punctuation() {
        return true;
    }
    matches!(ch,
        '，' | '。' | '、' | '；' | '：' | '？' | '！' | '“' | '”' | '‘' | '’'
            | '（' | '）' | '《' | '》' | '【' | '】' | '—' | '…' | '·' | '～' | '￥'
    )
}

const STOPWORDS: &[&str] = &[
    "的", "了", "是", "在", "我", "你", "他", "她", "它", "们", "这", "那", "个", "和", "与",
    "或", "但", "就", "都", "而", "及", "等", "不", "也", "很", "吗", "呢", "啊", "吧",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextPieces {
    pub classes: ClassCounts,
    pub chars: Vec<String>,
    pub words: Vec<String>,
}

/// `segment` 打开时，中文按相邻二字、三字计数，英文按连续字母成词。
/// 一次提交如果整段都是 8 个及以上的英文或数字，不把它当成一个词存下来。
/// 这挡不住密码进入单字频率，设置页会说明这件事。
pub fn pieces(text: &str, segment: bool) -> TextPieces {
    let classes = classify(text);
    let mut chars = Vec::new();
    let mut words = Vec::new();
    for ch in text.chars() {
        if !ch.is_whitespace() && !ch.is_control() {
            chars.push(ch.to_string());
        }
    }
    if segment {
        let whole_latin = text.chars().all(|ch| ch.is_ascii_alphanumeric()) && text.chars().count() >= 8;
        let mut han_run = String::new();
        let mut lat_run = String::new();
        let flush_han = |run: &str, words: &mut Vec<String>| {
            let cs: Vec<char> = run.chars().collect();
            for n in [2usize, 3] {
                if cs.len() < n {
                    continue;
                }
                for i in 0..=cs.len() - n {
                    let w: String = cs[i..i + n].iter().collect();
                    if !STOPWORDS.contains(&w.as_str()) {
                        words.push(w);
                    }
                }
            }
        };
        let flush_lat = |run: &str, words: &mut Vec<String>, blocked: bool| {
            if blocked || run.chars().count() < 2 || run.chars().count() > 24 {
                return;
            }
            let lower = run.to_ascii_lowercase();
            if !lower.chars().all(|ch| ch.is_ascii_alphabetic()) {
                return;
            }
            words.push(lower);
        };
        for ch in text.chars() {
            if is_han(ch) {
                flush_lat(&lat_run, &mut words, whole_latin);
                lat_run.clear();
                han_run.push(ch);
            } else if ch.is_ascii_alphabetic() {
                flush_han(&han_run, &mut words);
                han_run.clear();
                lat_run.push(ch);
            } else {
                flush_han(&han_run, &mut words);
                flush_lat(&lat_run, &mut words, whole_latin);
                han_run.clear();
                lat_run.clear();
            }
        }
        flush_han(&han_run, &mut words);
        flush_lat(&lat_run, &mut words, whole_latin);
    }
    TextPieces { classes, chars, words }
}

pub struct Ring {
    ptr: *mut u8,
    len: usize,
}

unsafe impl Send for Ring {}

impl Ring {
    /// 调用方保证 `ptr` 指向至少 [`map_len`] 字节、并且在进程间共享。
    ///
    /// # Safety
    /// `ptr` 必须非空、对齐满足字节访问，且在 `Ring` 生命周期内有效并至少包含 `len` 字节。
    pub unsafe fn from_ptr(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }

    pub fn init_if_needed(&self) {
        unsafe {
            let magic = self.ptr.add(OFF_MAGIC) as *mut u32;
            if magic.read_unaligned() == MAGIC && (self.ptr.add(OFF_VERSION) as *mut u32).read_unaligned() == VERSION {
                return;
            }
            std::ptr::write_bytes(self.ptr, 0, self.len.min(map_len()));
            magic.write_unaligned(MAGIC);
            (self.ptr.add(OFF_VERSION) as *mut u32).write_unaligned(VERSION);
        }
    }

    pub fn mark_hello(&self) {
        self.atom32(OFF_HELLO).store(1, Ordering::Release);
    }

    pub fn hello(&self) -> bool {
        self.atom32(OFF_HELLO).load(Ordering::Acquire) == 1
    }

    pub fn push(&self, bytes: &[u8]) -> bool {
        if bytes.is_empty() || self.len < map_len() {
            return false;
        }
        let n = bytes.len().min(SLOT_BYTES);
        let head = self.atom32(OFF_HEAD);
        let tail = self.atom32(OFF_TAIL);
        loop {
            let t = tail.load(Ordering::Relaxed);
            let h = head.load(Ordering::Acquire);
            if t.wrapping_sub(h) >= SLOTS {
                self.atom64(OFF_DROPPED).fetch_add(1, Ordering::Relaxed);
                return false;
            }
            if tail
                .compare_exchange_weak(t, t.wrapping_add(1), Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                unsafe {
                    let slot = self.slot(t);
                    std::ptr::write_bytes(slot, 0, SLOT_STRIDE);
                    (slot.add(4) as *mut u32).write_unaligned(n as u32);
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), slot.add(8), n);
                    AtomicU32::from_ptr(slot as *mut u32).store(t.wrapping_add(1), Ordering::Release);
                }
                self.atom64(OFF_ACCEPTED).fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
    }

    pub fn pop(&self, out: &mut [u8]) -> Option<usize> {
        let head = self.atom32(OFF_HEAD);
        let h = head.load(Ordering::Relaxed);
        let seq = unsafe { AtomicU32::from_ptr(self.slot(h) as *mut u32) };
        if seq.load(Ordering::Acquire) != h.wrapping_add(1) {
            return None;
        }
        let n = unsafe { (self.slot(h).add(4) as *const u32).read_unaligned() as usize };
        let n = n.min(SLOT_BYTES).min(out.len());
        unsafe {
            std::ptr::copy_nonoverlapping(self.slot(h).add(8), out.as_mut_ptr(), n);
            std::ptr::write_bytes(self.slot(h), 0, SLOT_STRIDE);
        }
        head.store(h.wrapping_add(1), Ordering::Release);
        Some(n)
    }

    pub fn accepted(&self) -> u64 {
        self.atom64(OFF_ACCEPTED).load(Ordering::Relaxed)
    }

    pub fn dropped(&self) -> u64 {
        self.atom64(OFF_DROPPED).load(Ordering::Relaxed)
    }

    fn atom32(&self, off: usize) -> &AtomicU32 {
        unsafe { AtomicU32::from_ptr(self.ptr.add(off) as *mut u32) }
    }

    fn atom64(&self, off: usize) -> &AtomicU64 {
        unsafe { AtomicU64::from_ptr(self.ptr.add(off) as *mut u64) }
    }

    unsafe fn slot(&self, index: u32) -> *mut u8 {
        self.ptr.add(HEADER + (index % SLOTS) as usize * SLOT_STRIDE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nihao_is_two_han_not_five_letters() {
        let c = classify("你好");
        assert_eq!(c.han, 2);
        assert_eq!(c.latin, 0);
        assert_eq!(c.chars(), 2);
    }

    #[test]
    fn mixed_commit() {
        let c = classify("你好, Rust 3!");
        assert_eq!(c.han, 2);
        assert_eq!(c.latin, 4);
        assert_eq!(c.digit, 1);
        assert!(c.punct >= 2);
        assert!(c.space >= 1);
    }

    #[test]
    fn words_are_ngrams_not_the_whole_sentence() {
        let p = pieces("你好世界", true);
        assert!(p.words.contains(&"你好".to_string()));
        assert!(p.words.contains(&"好世".to_string()));
        assert!(p.words.contains(&"世界".to_string()));
        assert!(p.words.contains(&"你好世".to_string()));
        assert!(!p.words.iter().any(|w| w == "你好世界"));
    }

    #[test]
    fn stopword_unigram_stays_char_but_not_a_word() {
        let p = pieces("的的", true);
        assert!(p.chars.iter().any(|c| c == "的"));
        assert!(!p.words.iter().any(|w| w == "的"));
    }

    #[test]
    fn long_single_token_is_not_stored_as_a_word() {
        let p = pieces("S3cretPass", true);
        assert_eq!(p.classes.latin, 9);
        assert_eq!(p.classes.digit, 1);
        assert!(p.words.is_empty());
    }

    #[test]
    fn segment_off_keeps_chars_only() {
        let p = pieces("你好", false);
        assert!(p.words.is_empty());
        assert_eq!(p.chars.len(), 2);
    }

    fn aligned() -> Vec<u64> {
        vec![0u64; map_len().div_ceil(8)]
    }

    #[test]
    fn ring_roundtrip_and_clears_slot() {
        let mut buf = aligned();
        let ring = unsafe { Ring::from_ptr(buf.as_mut_ptr() as *mut u8, buf.len() * 8) };
        ring.init_if_needed();
        assert!(ring.push("你好".as_bytes()));
        let mut out = [0u8; 32];
        let n = ring.pop(&mut out).unwrap();
        assert_eq!(&out[..n], "你好".as_bytes());
        assert!(ring.pop(&mut out).is_none());
        assert_eq!(ring.accepted(), 1);
    }

    #[test]
    fn ring_drops_when_full() {
        let mut buf = aligned();
        let ring = unsafe { Ring::from_ptr(buf.as_mut_ptr() as *mut u8, buf.len() * 8) };
        ring.init_if_needed();
        for i in 0..SLOTS {
            assert!(ring.push(&[i as u8]), "{i}");
        }
        assert!(!ring.push(b"x"));
        assert_eq!(ring.dropped(), 1);
    }

    #[test]
    fn ring_wraparound_preserves_order_and_clears() {
        let mut buf = aligned();
        let ring = unsafe { Ring::from_ptr(buf.as_mut_ptr() as *mut u8, buf.len() * 8) };
        ring.init_if_needed();
        let mut out = [0u8; 64];
        for round in 0..200u32 {
            let payload = format!("commit-{round}");
            assert!(ring.push(payload.as_bytes()));
            let n = ring.pop(&mut out).expect("pop after push");
            assert_eq!(&out[..n], payload.as_bytes());
        }
        assert_eq!(ring.accepted(), 200);
        assert_eq!(ring.dropped(), 0);
        assert!(ring.pop(&mut out).is_none());
    }
}
