//! SQLite 只保存已经加总的计数。

use crate::aggregate::{HourBucket, PeakRow};
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;
use typescope_proto::ClassCounts;

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = open_existing(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS meta (k TEXT PRIMARY KEY, v TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS hourly (
            day INTEGER NOT NULL, hour INTEGER NOT NULL,
            physical INTEGER NOT NULL DEFAULT 0, repeats INTEGER NOT NULL DEFAULT 0,
            injected INTEGER NOT NULL DEFAULT 0, backspace INTEGER NOT NULL DEFAULT 0,
            delete_key INTEGER NOT NULL DEFAULT 0, space_key INTEGER NOT NULL DEFAULT 0,
            enter_key INTEGER NOT NULL DEFAULT 0, shift_key INTEGER NOT NULL DEFAULT 0,
            ctrl_key INTEGER NOT NULL DEFAULT 0, alt_key INTEGER NOT NULL DEFAULT 0,
            active_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (day, hour)
        );
        CREATE TABLE IF NOT EXISTS key_counts (
            day INTEGER NOT NULL, vk INTEGER NOT NULL, downs INTEGER NOT NULL,
            PRIMARY KEY (day, vk)
        );
        CREATE TABLE IF NOT EXISTS intervals (
            day INTEGER NOT NULL, bucket INTEGER NOT NULL, n INTEGER NOT NULL,
            PRIMARY KEY (day, bucket)
        );
        CREATE TABLE IF NOT EXISTS shortcuts (
            day INTEGER NOT NULL, mods INTEGER NOT NULL, vk INTEGER NOT NULL, n INTEGER NOT NULL,
            PRIMARY KEY (day, mods, vk)
        );
        CREATE TABLE IF NOT EXISTS classes (
            day INTEGER NOT NULL, hour INTEGER NOT NULL,
            han INTEGER NOT NULL DEFAULT 0, latin INTEGER NOT NULL DEFAULT 0,
            digit INTEGER NOT NULL DEFAULT 0, punct INTEGER NOT NULL DEFAULT 0,
            space_ch INTEGER NOT NULL DEFAULT 0, other_ch INTEGER NOT NULL DEFAULT 0,
            commits INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (day, hour)
        );
        CREATE TABLE IF NOT EXISTS char_freq (
            day INTEGER NOT NULL, ch TEXT NOT NULL, n INTEGER NOT NULL,
            PRIMARY KEY (day, ch)
        );
        CREATE TABLE IF NOT EXISTS word_freq (
            day INTEGER NOT NULL, word TEXT NOT NULL, n INTEGER NOT NULL,
            PRIMARY KEY (day, word)
        );
        CREATE TABLE IF NOT EXISTS app_hourly (
            day INTEGER NOT NULL, hour INTEGER NOT NULL, exe TEXT NOT NULL,
            physical INTEGER NOT NULL DEFAULT 0, active_ms INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (day, hour, exe)
        );
        CREATE TABLE IF NOT EXISTS daily_peaks (
            day INTEGER PRIMARY KEY,
            best_10 INTEGER NOT NULL DEFAULT 0,
            best_30 INTEGER NOT NULL DEFAULT 0,
            best_60 INTEGER NOT NULL DEFAULT 0,
            sessions INTEGER NOT NULL DEFAULT 0,
            longest_ms INTEGER NOT NULL DEFAULT 0,
            first_minute INTEGER NOT NULL DEFAULT -1,
            last_minute INTEGER NOT NULL DEFAULT -1
        );
        ",
    )?;
    Ok(conn)
}

pub fn open_existing(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let conn = Connection::open(path)?;
    conn.busy_timeout(std::time::Duration::from_millis(1500))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "wal_autocheckpoint", 200)?;
    Ok(conn)
}

pub fn wal_checkpoint(conn: &Connection) {
    let _ = conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);");
}

pub fn meta_get(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT v FROM meta WHERE k=?1", [key], |r| r.get(0)).optional()
}

pub fn meta_set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute("INSERT INTO meta(k,v) VALUES(?1,?2) ON CONFLICT(k) DO UPDATE SET v=excluded.v", (key, value))?;
    Ok(())
}

pub fn flush_hours(conn: &Connection, rows: &[((u32, u8), HourBucket)]) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO hourly(day,hour,physical,repeats,injected,backspace,delete_key,space_key,enter_key,shift_key,ctrl_key,alt_key,active_ms)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
         ON CONFLICT(day,hour) DO UPDATE SET
            physical=physical+excluded.physical, repeats=repeats+excluded.repeats,
            injected=injected+excluded.injected, backspace=backspace+excluded.backspace,
            delete_key=delete_key+excluded.delete_key, space_key=space_key+excluded.space_key,
            enter_key=enter_key+excluded.enter_key, shift_key=shift_key+excluded.shift_key,
            ctrl_key=ctrl_key+excluded.ctrl_key, alt_key=alt_key+excluded.alt_key,
            active_ms=active_ms+excluded.active_ms",
    )?;
    for ((day, hour), b) in rows {
        stmt.execute((day, hour, b.physical, b.repeats, b.injected, b.backspace, b.delete_key, b.space, b.enter, b.shift, b.ctrl, b.alt, b.active_ms))?;
    }
    Ok(())
}

pub fn flush_keys(conn: &Connection, rows: &[(u32, [u32; 256])]) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO key_counts(day,vk,downs) VALUES(?1,?2,?3)
         ON CONFLICT(day,vk) DO UPDATE SET downs=downs+excluded.downs",
    )?;
    for (day, counts) in rows {
        for (vk, n) in counts.iter().enumerate() {
            if *n > 0 {
                stmt.execute((day, vk as u16, n))?;
            }
        }
    }
    Ok(())
}

pub fn flush_intervals(conn: &Connection, rows: &[(u32, [u32; 12])]) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO intervals(day,bucket,n) VALUES(?1,?2,?3)
         ON CONFLICT(day,bucket) DO UPDATE SET n=n+excluded.n",
    )?;
    for (day, buckets) in rows {
        for (i, n) in buckets.iter().enumerate() {
            if *n > 0 {
                stmt.execute((day, i as u32, n))?;
            }
        }
    }
    Ok(())
}

pub fn flush_shortcuts(conn: &Connection, rows: &[(u32, u8, u16, u32)]) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO shortcuts(day,mods,vk,n) VALUES(?1,?2,?3,?4)
         ON CONFLICT(day,mods,vk) DO UPDATE SET n=n+excluded.n",
    )?;
    for (day, mods, vk, n) in rows {
        stmt.execute((day, mods, vk, n))?;
    }
    Ok(())
}

pub fn flush_classes(conn: &Connection, day: u32, hour: u8, counts: ClassCounts, commits: u32) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO classes(day,hour,han,latin,digit,punct,space_ch,other_ch,commits)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)
         ON CONFLICT(day,hour) DO UPDATE SET
            han=han+excluded.han, latin=latin+excluded.latin, digit=digit+excluded.digit,
            punct=punct+excluded.punct, space_ch=space_ch+excluded.space_ch,
            other_ch=other_ch+excluded.other_ch, commits=commits+excluded.commits",
        (day, hour, counts.han, counts.latin, counts.digit, counts.punct, counts.space, counts.other, commits),
    )?;
    Ok(())
}

pub fn flush_freq(conn: &Connection, table: &str, day: u32, items: &[(String, u32)]) -> rusqlite::Result<()> {
    let col = if table == "word_freq" { "word" } else { "ch" };
    let sql = format!(
        "INSERT INTO {table}(day,{col},n) VALUES(?1,?2,?3)
         ON CONFLICT(day,{col}) DO UPDATE SET n=n+excluded.n"
    );
    let mut stmt = conn.prepare(&sql)?;
    for (item, n) in items {
        stmt.execute((day, item, n))?;
    }
    Ok(())
}

pub fn flush_freq_batch(conn: &Connection, table: &str, items: &[(u32, String, u32)]) -> rusqlite::Result<()> {
    if items.is_empty() {
        return Ok(());
    }
    let col = if table == "word_freq" { "word" } else { "ch" };
    let sql = format!(
        "INSERT INTO {table}(day,{col},n) VALUES(?1,?2,?3)
         ON CONFLICT(day,{col}) DO UPDATE SET n=n+excluded.n"
    );
    let mut stmt = conn.prepare(&sql)?;
    for (day, item, n) in items {
        stmt.execute((day, item, n))?;
    }
    Ok(())
}

pub fn flush_app(conn: &Connection, day: u32, hour: u8, exe: &str, physical: u32, active_ms: u64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO app_hourly(day,hour,exe,physical,active_ms) VALUES(?1,?2,?3,?4,?5)
         ON CONFLICT(day,hour,exe) DO UPDATE SET physical=physical+excluded.physical, active_ms=active_ms+excluded.active_ms",
        (day, hour, exe, physical, active_ms),
    )?;
    Ok(())
}

pub fn flush_peaks(conn: &Connection, day: u32, row: PeakRow) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO daily_peaks(day,best_10,best_30,best_60,sessions,longest_ms,first_minute,last_minute)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(day) DO UPDATE SET
            best_10=MAX(daily_peaks.best_10, excluded.best_10),
            best_30=MAX(daily_peaks.best_30, excluded.best_30),
            best_60=MAX(daily_peaks.best_60, excluded.best_60),
            sessions=daily_peaks.sessions+excluded.sessions,
            longest_ms=MAX(daily_peaks.longest_ms, excluded.longest_ms),
            first_minute=CASE
                WHEN daily_peaks.first_minute < 0 THEN excluded.first_minute
                WHEN excluded.first_minute < 0 THEN daily_peaks.first_minute
                ELSE MIN(daily_peaks.first_minute, excluded.first_minute)
            END,
            last_minute=CASE
                WHEN daily_peaks.last_minute < 0 THEN excluded.last_minute
                WHEN excluded.last_minute < 0 THEN daily_peaks.last_minute
                ELSE MAX(daily_peaks.last_minute, excluded.last_minute)
            END",
        (day, row.best_10, row.best_30, row.best_60, row.sessions, row.longest_ms, row.first_minute, row.last_minute),
    )?;
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct DayView {
    pub physical: u64,
    pub repeats: u64,
    pub injected: u64,
    pub backspace: u64,
    pub active_ms: u64,
    pub han: u64,
    pub latin: u64,
    pub digit: u64,
    pub punct: u64,
    pub commits: u64,
    pub best_10: u32,
    pub sessions: u32,
    pub longest_ms: u64,
    pub first_minute: i32,
    pub last_minute: i32,
}

pub fn day_view(conn: &Connection, day: u32) -> rusqlite::Result<DayView> {
    let mut view = DayView { first_minute: -1, last_minute: -1, ..DayView::default() };
    conn.query_row(
        "SELECT COALESCE(SUM(physical),0), COALESCE(SUM(repeats),0), COALESCE(SUM(injected),0),
                COALESCE(SUM(backspace),0), COALESCE(SUM(active_ms),0)
         FROM hourly WHERE day=?1",
        [day],
        |r| {
            view.physical = r.get(0)?;
            view.repeats = r.get(1)?;
            view.injected = r.get(2)?;
            view.backspace = r.get(3)?;
            view.active_ms = r.get(4)?;
            Ok(())
        },
    )?;
    let _ = conn.query_row(
        "SELECT COALESCE(SUM(han),0), COALESCE(SUM(latin),0), COALESCE(SUM(digit),0),
                COALESCE(SUM(punct),0), COALESCE(SUM(commits),0)
         FROM classes WHERE day=?1",
        [day],
        |r| {
            view.han = r.get(0)?;
            view.latin = r.get(1)?;
            view.digit = r.get(2)?;
            view.punct = r.get(3)?;
            view.commits = r.get(4)?;
            Ok(())
        },
    );
    if let Ok(Some((b, s, l, first, last))) = conn
        .query_row(
            "SELECT best_10, sessions, longest_ms, first_minute, last_minute FROM daily_peaks WHERE day=?1",
            [day],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()
    {
        view.best_10 = b;
        view.sessions = s;
        view.longest_ms = l;
        view.first_minute = first;
        view.last_minute = last;
    }
    Ok(view)
}

pub fn hourly_physical(conn: &Connection, day: u32) -> rusqlite::Result<[u64; 24]> {
    let mut out = [0u64; 24];
    let mut stmt = conn.prepare("SELECT hour, physical FROM hourly WHERE day=?1")?;
    let rows = stmt.query_map([day], |r| Ok((r.get::<_, u8>(0)?, r.get::<_, u64>(1)?)))?;
    for row in rows {
        let (h, n) = row?;
        if (h as usize) < 24 {
            out[h as usize] = n;
        }
    }
    Ok(out)
}

pub fn top_keys(conn: &Connection, day: u32, limit: usize) -> rusqlite::Result<Vec<(u16, u64)>> {
    let mut stmt = conn.prepare("SELECT vk, downs FROM key_counts WHERE day=?1 ORDER BY downs DESC LIMIT ?2")?;
    let rows = stmt.query_map((day, limit as i64), |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

pub fn top_words(conn: &Connection, day: u32, limit: usize) -> rusqlite::Result<Vec<(String, u64)>> {
    let mut stmt = conn.prepare("SELECT word, n FROM word_freq WHERE day=?1 ORDER BY n DESC LIMIT ?2")?;
    let rows = stmt.query_map((day, limit as i64), |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HourDetail {
    pub physical: u64,
    pub active_ms: u64,
    pub backspace: u64,
    pub han: u64,
    pub latin: u64,
}

pub fn hour_details(conn: &Connection, day: u32) -> rusqlite::Result<[HourDetail; 24]> {
    let mut out = [HourDetail::default(); 24];
    let mut stmt = conn.prepare("SELECT hour, physical, active_ms, backspace FROM hourly WHERE day=?1")?;
    let rows = stmt.query_map([day], |r| Ok((r.get::<_, u8>(0)?, r.get::<_, u64>(1)?, r.get::<_, u64>(2)?, r.get::<_, u64>(3)?)))?;
    for row in rows {
        let (h, physical, active_ms, backspace) = row?;
        if (h as usize) < 24 {
            out[h as usize].physical = physical;
            out[h as usize].active_ms = active_ms;
            out[h as usize].backspace = backspace;
        }
    }
    drop(stmt);
    let mut stmt = conn.prepare("SELECT hour, han, latin FROM classes WHERE day=?1")?;
    let rows = stmt.query_map([day], |r| Ok((r.get::<_, u8>(0)?, r.get::<_, u64>(1)?, r.get::<_, u64>(2)?)))?;
    for row in rows {
        let (h, han, latin) = row?;
        if (h as usize) < 24 {
            out[h as usize].han = han;
            out[h as usize].latin = latin;
        }
    }
    Ok(out)
}

pub fn daily_totals(conn: &Connection) -> rusqlite::Result<Vec<(u32, u64, u64)>> {
    let mut stmt = conn.prepare(
        "SELECT day, COALESCE(SUM(physical),0), COALESCE(SUM(active_ms),0)
         FROM hourly GROUP BY day ORDER BY day",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    rows.collect()
}

pub fn lifetime(conn: &Connection) -> rusqlite::Result<(u64, u64, u64, u64, u64)> {
    let (physical, active) = conn.query_row(
        "SELECT COALESCE(SUM(physical),0), COALESCE(SUM(active_ms),0) FROM hourly",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let (han, latin, commits) = conn.query_row(
        "SELECT COALESCE(SUM(han),0), COALESCE(SUM(latin),0), COALESCE(SUM(commits),0) FROM classes",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok((physical, active, han, latin, commits))
}

pub fn all_key_counts(conn: &Connection, day: u32) -> rusqlite::Result<Vec<(u16, u64)>> {
    let mut stmt = conn.prepare("SELECT vk, downs FROM key_counts WHERE day=?1")?;
    let rows = stmt.query_map([day], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

pub fn top_chars(conn: &Connection, day: u32, limit: usize) -> rusqlite::Result<Vec<(String, u64)>> {
    let mut stmt = conn.prepare("SELECT ch, n FROM char_freq WHERE day=?1 ORDER BY n DESC LIMIT ?2")?;
    let rows = stmt.query_map((day, limit as i64), |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

pub fn top_shortcuts(conn: &Connection, day: u32, limit: usize) -> rusqlite::Result<Vec<(u8, u16, u64)>> {
    let mut stmt = conn.prepare("SELECT mods, vk, n FROM shortcuts WHERE day=?1 ORDER BY n DESC LIMIT ?2")?;
    let rows = stmt.query_map((day, limit as i64), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    rows.collect()
}

pub fn top_apps(conn: &Connection, day: u32, limit: usize) -> rusqlite::Result<Vec<(String, u64, u64)>> {
    let mut stmt = conn.prepare(
        "SELECT exe, SUM(physical), SUM(active_ms) FROM app_hourly WHERE day=?1
         GROUP BY exe ORDER BY SUM(physical) DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map((day, limit as i64), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    rows.collect()
}

pub fn interval_counts(conn: &Connection, day: u32) -> rusqlite::Result<[u64; 12]> {
    let mut out = [0u64; 12];
    let mut stmt = conn.prepare("SELECT bucket, n FROM intervals WHERE day=?1")?;
    let rows = stmt.query_map([day], |r| Ok((r.get::<_, u8>(0)?, r.get::<_, u64>(1)?)))?;
    for row in rows {
        let (bucket, n) = row?;
        if (bucket as usize) < 12 {
            out[bucket as usize] = n;
        }
    }
    Ok(out)
}

pub fn recent_hourly(conn: &Connection, since_day: u32) -> rusqlite::Result<Vec<(u32, u8, u64)>> {
    let mut stmt = conn.prepare("SELECT day, hour, physical FROM hourly WHERE day>=?1")?;
    let rows = stmt.query_map([since_day], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    rows.collect()
}

pub fn clear_all(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "DELETE FROM hourly; DELETE FROM key_counts; DELETE FROM intervals; DELETE FROM shortcuts;
         DELETE FROM classes; DELETE FROM char_freq; DELETE FROM word_freq; DELETE FROM app_hourly;
         DELETE FROM daily_peaks;",
    )?;
    wal_checkpoint(conn);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::{average_kpm, Engine, KeySample, Mods};

    #[test]
    fn flush_then_read_day() {
        let path = std::env::temp_dir().join(format!("typescope-test-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let conn = open(&path).unwrap();
        let mut engine = Engine::default();
        engine.observe(20261031, 8, KeySample { vk: 0x41, down: true, repeat: false, injected: false, at_ms: 0 }, Mods::default());
        engine.observe(20261031, 8, KeySample { vk: 0x42, down: true, repeat: false, injected: false, at_ms: 100 }, Mods::default());
        let hours = engine.take_hours();
        let keys = engine.take_keys();
        flush_hours(&conn, &hours).unwrap();
        flush_keys(&conn, &keys).unwrap();
        let view = day_view(&conn, 20261031).unwrap();
        assert_eq!(view.physical, 2);
        assert_eq!(view.active_ms, 100);
        assert_eq!(average_kpm(view.physical as u32, view.active_ms), Some(1200));
        let _ = std::fs::remove_file(&path);
    }
}
