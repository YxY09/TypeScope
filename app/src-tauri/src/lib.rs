//! TypeScope 看板的 Tauri 后端。只读同一个 SQLite，写操作只碰 meta 表。

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use rusqlite::Connection;
use serde::Serialize;
use typescope::{dates, platform, store};

static DB_INIT_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

#[derive(Serialize)]
struct HourDto {
    physical: u64,
    active_ms: u64,
    backspace: u64,
    han: u64,
    latin: u64,
}

#[derive(Serialize)]
struct ViewDto {
    physical: u64,
    repeats: u64,
    injected: u64,
    backspace: u64,
    active_ms: u64,
    han: u64,
    latin: u64,
    digit: u64,
    punct: u64,
    commits: u64,
    best_10: u32,
    sessions: u32,
    longest_ms: u64,
    first_minute: i32,
    last_minute: i32,
}

#[derive(Serialize)]
struct KeyDto {
    vk: u16,
    n: u64,
}

#[derive(Serialize)]
struct CountDto {
    k: String,
    n: u64,
}

#[derive(Serialize)]
struct ShortcutDto {
    mods: u8,
    vk: u16,
    n: u64,
}

#[derive(Serialize)]
struct AppDto {
    exe: String,
    physical: u64,
    active_ms: u64,
}

#[derive(Serialize)]
struct DayDto {
    day: u32,
    physical: u64,
    active_ms: u64,
}

#[derive(Serialize)]
struct RecentDto {
    day: u32,
    hour: u8,
    physical: u64,
}

#[derive(Serialize)]
struct LifeDto {
    physical: u64,
    active_ms: u64,
    han: u64,
    latin: u64,
    commits: u64,
}

#[derive(Serialize)]
struct FlagsDto {
    paused: bool,
    text_stats: bool,
    segment: bool,
    app_stats: bool,
    rime_hello: bool,
}

#[derive(Serialize)]
struct Snapshot {
    day: u32,
    view: ViewDto,
    hours: Vec<HourDto>,
    keys: Vec<KeyDto>,
    chars: Vec<CountDto>,
    words: Vec<CountDto>,
    shortcuts: Vec<ShortcutDto>,
    apps: Vec<AppDto>,
    intervals: Vec<u64>,
    days: Vec<DayDto>,
    recent: Vec<RecentDto>,
    lifetime: LifeDto,
    streak: u32,
    flags: FlagsDto,
    idle_ms: u64,
    rime_accepted: u64,
    agent_on: bool,
    autostart: bool,
}

fn open_db() -> Result<Connection, String> {
    let path = platform::db_path();
    let cache = DB_INIT_PATH.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let already_init = guard.as_ref() == Some(&path) && path.exists();
    if already_init {
        store::open_existing(&path).map_err(|error| error.to_string())
    } else {
        let conn = store::open(&path).map_err(|error| error.to_string())?;
        *guard = Some(path);
        Ok(conn)
    }
}

fn flag(conn: &Connection, key: &str) -> bool {
    store::meta_get(conn, key).ok().flatten().as_deref() == Some("1")
}

fn streak_of(days: &[(u32, u64, u64)], today: u32) -> u32 {
    let mut cursor = if days.iter().any(|(day, n, _)| *day == today && *n > 0) {
        today
    } else {
        dates::add_days(today, -1)
    };
    let mut count = 0;
    while days.iter().any(|(day, n, _)| *day == cursor && *n > 0) {
        count += 1;
        cursor = dates::add_days(cursor, -1);
        if count > 4000 {
            break;
        }
    }
    count
}

#[tauri::command]
fn snapshot(day: Option<u32>) -> Result<Snapshot, String> {
    let conn = open_db()?;
    let (today, _, _) = platform::local_stamp();
    let day = day.unwrap_or(today);
    let view = store::day_view(&conn, day).map_err(|error| error.to_string())?;
    let hours = store::hour_details(&conn, day).map_err(|error| error.to_string())?;
    let keys = store::all_key_counts(&conn, day).map_err(|error| error.to_string())?;
    let chars = store::top_chars(&conn, day, 30).map_err(|error| error.to_string())?;
    let words = store::top_words(&conn, day, 30).map_err(|error| error.to_string())?;
    let shortcuts = store::top_shortcuts(&conn, day, 10).map_err(|error| error.to_string())?;
    let apps = store::top_apps(&conn, day, 10).map_err(|error| error.to_string())?;
    let intervals = store::interval_counts(&conn, day).map_err(|error| error.to_string())?;
    let days = store::daily_totals(&conn).map_err(|error| error.to_string())?;
    let recent = store::recent_hourly(&conn, dates::add_days(day, -84)).map_err(|error| error.to_string())?;
    let lifetime = store::lifetime(&conn).map_err(|error| error.to_string())?;
    let idle_ms = store::meta_get(&conn, "idle_ms")
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
        .unwrap_or(typescope::aggregate::IDLE_MS_DEFAULT);
    let rime_accepted = store::meta_get(&conn, "rime_accepted")
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    Ok(Snapshot {
        day,
        streak: streak_of(&days, day),
        view: ViewDto {
            physical: view.physical,
            repeats: view.repeats,
            injected: view.injected,
            backspace: view.backspace,
            active_ms: view.active_ms,
            han: view.han,
            latin: view.latin,
            digit: view.digit,
            punct: view.punct,
            commits: view.commits,
            best_10: view.best_10,
            sessions: view.sessions,
            longest_ms: view.longest_ms,
            first_minute: view.first_minute,
            last_minute: view.last_minute,
        },
        hours: hours
            .iter()
            .map(|hour| HourDto {
                physical: hour.physical,
                active_ms: hour.active_ms,
                backspace: hour.backspace,
                han: hour.han,
                latin: hour.latin,
            })
            .collect(),
        keys: keys.iter().map(|(vk, n)| KeyDto { vk: *vk, n: *n }).collect(),
        chars: chars.iter().map(|(k, n)| CountDto { k: k.clone(), n: *n }).collect(),
        words: words.iter().map(|(k, n)| CountDto { k: k.clone(), n: *n }).collect(),
        shortcuts: shortcuts.iter().map(|(mods, vk, n)| ShortcutDto { mods: *mods, vk: *vk, n: *n }).collect(),
        apps: apps.iter().map(|(exe, physical, active_ms)| AppDto { exe: exe.clone(), physical: *physical, active_ms: *active_ms }).collect(),
        intervals: intervals.to_vec(),
        days: days.iter().map(|(day, physical, active)| DayDto { day: *day, physical: *physical, active_ms: *active }).collect(),
        recent: recent.iter().map(|(day, hour, physical)| RecentDto { day: *day, hour: *hour, physical: *physical }).collect(),
        lifetime: LifeDto {
            physical: lifetime.0,
            active_ms: lifetime.1,
            han: lifetime.2,
            latin: lifetime.3,
            commits: lifetime.4,
        },
        flags: FlagsDto {
            paused: flag(&conn, "paused"),
            text_stats: flag(&conn, "text_stats"),
            segment: flag(&conn, "segment"),
            app_stats: flag(&conn, "app_stats"),
            rime_hello: flag(&conn, "rime_hello"),
        },
        idle_ms,
        rime_accepted,
        agent_on: platform::agent_running(),
        autostart: platform::autostart_enabled(),
    })
}

#[tauri::command]
fn set_flag(key: String, value: String) -> Result<(), String> {
    let conn = open_db()?;
    store::meta_set(&conn, &key, &value).map_err(|error| error.to_string())?;
    send_command("reload".to_string())
}

#[tauri::command]
fn set_idle(ms: u64) -> Result<(), String> {
    let conn = open_db()?;
    store::meta_set(&conn, "idle_ms", &ms.to_string()).map_err(|error| error.to_string())?;
    send_command("reload".to_string())
}

#[tauri::command]
fn send_command(command: String) -> Result<(), String> {
    let conn = open_db()?;
    store::meta_set(&conn, "command", &command).map_err(|error| error.to_string())?;
    if command == "pause" {
        let _ = store::meta_set(&conn, "paused", "1");
    }
    if command == "resume" {
        let _ = store::meta_set(&conn, "paused", "0");
    }
    platform::signal_control();
    Ok(())
}

#[tauri::command]
fn set_autostart(on: bool) -> Result<bool, String> {
    Ok(platform::set_autostart(on))
}

#[tauri::command]
fn open_exclude() -> Result<(), String> {
    platform::ensure_exclude_file();
    platform::open_path(&platform::exclude_path());
    Ok(())
}

#[tauri::command]
fn open_data_dir() -> Result<(), String> {
    platform::open_path(&platform::data_dir());
    Ok(())
}

#[tauri::command]
fn spawn_agent() -> Result<bool, String> {
    if platform::agent_running() {
        return Ok(true);
    }
    Ok(platform::spawn_sibling("typescope-agent.exe"))
}

#[tauri::command]
fn install_rime() -> Result<String, String> {
    platform::install_rime()
}

#[tauri::command]
fn window_control(window: tauri::Window, action: String) -> Result<bool, String> {
    match action.as_str() {
        "minimize" => {
            let _ = window.minimize();
            Ok(false)
        }
        "toggle_max" => {
            let is_max = window.is_maximized().unwrap_or(false);
            if is_max {
                let _ = window.unmaximize();
                Ok(false)
            } else {
                let _ = window.maximize();
                Ok(true)
            }
        }
        "is_max" => Ok(window.is_maximized().unwrap_or(false)),
        "close" => {
            let _ = window.close();
            Ok(false)
        }
        "drag" => {
            let _ = window.start_dragging();
            Ok(false)
        }
        _ => Ok(false),
    }
}

#[derive(Serialize)]
struct DaySeries {
    day: u32,
    physical: u64,
    active_ms: u64,
    backspace: u64,
    delete_key: u64,
    space_key: u64,
    enter_key: u64,
    han: u64,
    latin: u64,
    digit: u64,
    punct: u64,
    commits: u64,
    best_10: u32,
    best_30: u32,
    best_60: u32,
    sessions: u32,
    longest_ms: u64,
    first_minute: i32,
    last_minute: i32,
}

#[derive(Serialize)]
struct KeyTrendPoint {
    day: u32,
    n: u64,
}

#[derive(Serialize)]
struct RangeReport {
    start: u32,
    end: u32,
    days: Vec<DaySeries>,
    top_keys: Vec<KeyDto>,
    top_chars: Vec<CountDto>,
    top_words: Vec<CountDto>,
    top_shortcuts: Vec<ShortcutDto>,
    top_apps: Vec<AppDto>,
    intervals: Vec<u64>,
    hour_grid: Vec<Vec<u64>>,
    calendar_days: Vec<DayDto>,
    delete_total: u64,
    space_total: u64,
    enter_total: u64,
    record_10: u32,
    record_30: u32,
    record_60: u32,
    record_longest_ms: u64,
}

#[tauri::command]
fn range_report(start: u32, end: u32) -> Result<RangeReport, String> {
    let conn = open_db()?;
    let mut days: Vec<DaySeries> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT day, COALESCE(SUM(physical),0), COALESCE(SUM(active_ms),0), COALESCE(SUM(backspace),0),
                        COALESCE(SUM(delete_key),0), COALESCE(SUM(space_key),0), COALESCE(SUM(enter_key),0)
                 FROM hourly WHERE day BETWEEN ?1 AND ?2 GROUP BY day",
            )
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map([start, end], |r| {
                Ok(DaySeries {
                    day: r.get(0)?,
                    physical: r.get(1)?,
                    active_ms: r.get(2)?,
                    backspace: r.get(3)?,
                    delete_key: r.get(4)?,
                    space_key: r.get(5)?,
                    enter_key: r.get(6)?,
                    han: 0,
                    latin: 0,
                    digit: 0,
                    punct: 0,
                    commits: 0,
                    best_10: 0,
                    best_30: 0,
                    best_60: 0,
                    sessions: 0,
                    longest_ms: 0,
                    first_minute: -1,
                    last_minute: -1,
                })
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            days.push(row.map_err(|error| error.to_string())?);
        }
    }
    {
        let mut stmt = conn
            .prepare(
                "SELECT day, COALESCE(SUM(han),0), COALESCE(SUM(latin),0), COALESCE(SUM(digit),0),
                        COALESCE(SUM(punct),0), COALESCE(SUM(commits),0)
                 FROM classes WHERE day BETWEEN ?1 AND ?2 GROUP BY day",
            )
            .map_err(|error| error.to_string())?;
        let rows = stmt.query_map([start, end], |r| {
            Ok((r.get::<_, u32>(0)?, r.get::<_, u64>(1)?, r.get::<_, u64>(2)?, r.get::<_, u64>(3)?, r.get::<_, u64>(4)?, r.get::<_, u64>(5)?))
        });
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                if let Some(day) = days.iter_mut().find(|d| d.day == row.0) {
                    day.han = row.1;
                    day.latin = row.2;
                    day.digit = row.3;
                    day.punct = row.4;
                    day.commits = row.5;
                }
            }
        }
    }
    {
        let mut stmt = conn
            .prepare(
                "SELECT day, best_10, best_30, best_60, sessions, longest_ms, first_minute, last_minute
                 FROM daily_peaks WHERE day BETWEEN ?1 AND ?2",
            )
            .map_err(|error| error.to_string())?;
        let mapped = stmt.query_map([start, end], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, u32>(2)?,
                r.get::<_, u32>(3)?,
                r.get::<_, u32>(4)?,
                r.get::<_, u64>(5)?,
                r.get::<_, i32>(6)?,
                r.get::<_, i32>(7)?,
            ))
        });
        if let Ok(rows) = mapped {
            for row in rows.flatten() {
                if let Some(day) = days.iter_mut().find(|d| d.day == row.0) {
                    day.best_10 = row.1;
                    day.best_30 = row.2;
                    day.best_60 = row.3;
                    day.sessions = row.4;
                    day.longest_ms = row.5;
                    day.first_minute = row.6;
                    day.last_minute = row.7;
                }
            }
        }
    }
    days.sort_by_key(|d| d.day);

    let top_keys = query_pairs(&conn, "SELECT vk, SUM(downs) FROM key_counts WHERE day BETWEEN ?1 AND ?2 GROUP BY vk ORDER BY 2 DESC LIMIT 150", start, end, |r| {
        Ok(KeyDto { vk: r.get(0)?, n: r.get(1)? })
    })?;
    let top_chars = query_pairs(&conn, "SELECT ch, SUM(n) FROM char_freq WHERE day BETWEEN ?1 AND ?2 GROUP BY ch ORDER BY 2 DESC LIMIT 40", start, end, |r| {
        Ok(CountDto { k: r.get(0)?, n: r.get(1)? })
    })?;
    let top_words = query_pairs(&conn, "SELECT word, SUM(n) FROM word_freq WHERE day BETWEEN ?1 AND ?2 GROUP BY word ORDER BY 2 DESC LIMIT 40", start, end, |r| {
        Ok(CountDto { k: r.get(0)?, n: r.get(1)? })
    })?;
    let top_shortcuts = query_triples(&conn, "SELECT mods, vk, SUM(n) FROM shortcuts WHERE day BETWEEN ?1 AND ?2 GROUP BY mods, vk ORDER BY 3 DESC LIMIT 10", start, end)?;
    let top_apps = query_apps(&conn, start, end)?;

    let intervals = query_intervals(&conn, start, end)?;
    let mut hour_grid: Vec<Vec<u64>> = vec![vec![0; 24]; 7];
    {
        let mut stmt = conn
            .prepare("SELECT day, hour, physical FROM hourly WHERE day BETWEEN ?1 AND ?2")
            .map_err(|error| error.to_string())?;
        let rows = stmt.query_map([start, end], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, u8>(1)?, r.get::<_, u64>(2)?)));
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                let week = dates::weekday_mon(row.0) as usize;
                if week < 7 && (row.1 as usize) < 24 {
                    hour_grid[week][row.1 as usize] += row.2;
                }
            }
        }
    }

    let (delete_total, space_total, enter_total): (u64, u64, u64) = conn
        .query_row(
            "SELECT COALESCE(SUM(delete_key),0), COALESCE(SUM(space_key),0), COALESCE(SUM(enter_key),0)
             FROM hourly WHERE day BETWEEN ?1 AND ?2",
            [start, end],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap_or((0, 0, 0));
    let (record_10, record_30, record_60, record_longest_ms): (u32, u32, u32, u64) = conn
        .query_row(
            "SELECT COALESCE(MAX(best_10),0), COALESCE(MAX(best_30),0), COALESCE(MAX(best_60),0), COALESCE(MAX(longest_ms),0) FROM daily_peaks",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap_or((0, 0, 0, 0));

    let calendar_days = store::daily_totals(&conn)
        .map(|rows| {
            rows.into_iter()
                .map(|(day, physical, active_ms)| DayDto {
                    day,
                    physical,
                    active_ms,
                })
                .collect()
        })
        .map_err(|error| error.to_string())?;

    Ok(RangeReport {
        start,
        end,
        days,
        top_keys,
        top_chars,
        top_words,
        top_shortcuts,
        top_apps,
        intervals,
        hour_grid,
        calendar_days,
        delete_total,
        space_total,
        enter_total,
        record_10,
        record_30,
        record_60,
        record_longest_ms,
    })
}

fn query_pairs<T, F>(conn: &Connection, sql: &str, start: u32, end: u32, map: F) -> Result<Vec<T>, String>
where
    F: Fn(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
{
    let mut stmt = conn.prepare(sql).map_err(|error| error.to_string())?;
    let rows = stmt.query_map([start, end], map).map_err(|error| error.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|error| error.to_string())?);
    }
    Ok(out)
}

fn query_triples(conn: &Connection, sql: &str, start: u32, end: u32) -> Result<Vec<ShortcutDto>, String> {
    let mut stmt = conn.prepare(sql).map_err(|error| error.to_string())?;
    let rows = stmt.query_map([start, end], |r| {
        Ok(ShortcutDto { mods: r.get(0)?, vk: r.get(1)?, n: r.get(2)? })
    });
    let mut out = Vec::new();
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            out.push(row);
        }
    }
    Ok(out)
}

fn query_apps(conn: &Connection, start: u32, end: u32) -> Result<Vec<AppDto>, String> {
    let mut stmt = conn
        .prepare("SELECT exe, SUM(physical), SUM(active_ms) FROM app_hourly WHERE day BETWEEN ?1 AND ?2 GROUP BY exe ORDER BY 2 DESC LIMIT 12")
        .map_err(|error| error.to_string())?;
    let rows = stmt.query_map([start, end], |r| {
        Ok(AppDto { exe: r.get(0)?, physical: r.get(1)?, active_ms: r.get(2)? })
    });
    let mut out = Vec::new();
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            out.push(row);
        }
    }
    Ok(out)
}

fn query_intervals(conn: &Connection, start: u32, end: u32) -> Result<Vec<u64>, String> {
    let mut stmt = conn
        .prepare("SELECT bucket, SUM(n) FROM intervals WHERE day BETWEEN ?1 AND ?2 GROUP BY bucket")
        .map_err(|error| error.to_string())?;
    let rows = stmt.query_map([start, end], |r| Ok((r.get::<_, u8>(0)?, r.get::<_, u64>(1)?)));
    let mut out = vec![0u64; 12];
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            if (row.0 as usize) < 12 {
                out[row.0 as usize] = row.1;
            }
        }
    }
    Ok(out)
}

#[tauri::command]
fn key_trend(vk: u16, start: u32, end: u32) -> Result<Vec<KeyTrendPoint>, String> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare("SELECT day, downs FROM key_counts WHERE vk=?1 AND day BETWEEN ?2 AND ?3 ORDER BY day")
        .map_err(|error| error.to_string())?;
    let rows = stmt.query_map(rusqlite::params![vk, start, end], |r| {
        Ok(KeyTrendPoint { day: r.get(0)?, n: r.get(1)? })
    });
    let mut out = Vec::new();
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            out.push(row);
        }
    }
    Ok(out)
}

#[tauri::command]
fn text_trend(table: String, key: String, start: u32, end: u32) -> Result<Vec<KeyTrendPoint>, String> {
    if table != "char_freq" && table != "word_freq" {
        return Err("不支持的表".to_string());
    }
    let conn = open_db()?;
    let column = if table == "char_freq" { "ch" } else { "word" };
    let sql = format!("SELECT day, n FROM {table} WHERE {column}=?1 AND day BETWEEN ?2 AND ?3 ORDER BY day");
    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt.query_map(rusqlite::params![key, start, end], |r| {
        Ok(KeyTrendPoint { day: r.get(0)?, n: r.get(1)? })
    });
    let mut out = Vec::new();
    if let Ok(rows) = rows {
        for row in rows.flatten() {
            out.push(row);
        }
    }
    Ok(out)
}

pub fn run() {
    let _app_lock = match platform::InstanceLock::acquire(platform::APP_MUTEX) {
        Some(lock) => lock,
        None => {
            let _ = platform::focus_existing_dashboard();
            return;
        }
    };
    if !platform::agent_running() {
        let _ = platform::spawn_sibling("typescope-agent.exe");
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            snapshot,
            range_report,
            key_trend,
            text_trend,
            set_flag,
            set_idle,
            send_command,
            set_autostart,
            open_exclude,
            open_data_dir,
            spawn_agent,
            install_rime,
            window_control
        ])
        .run(tauri::generate_context!())
        .expect("TypeScope 看板启动失败");
}
