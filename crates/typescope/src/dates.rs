//! 本地日期。数据库里的 day 是 YYYYMMDD，不保存时间戳序列。

pub fn ymd(day: u32) -> (i32, u32, u32) {
    let y = (day / 10_000) as i32;
    let m = (day / 100) % 100;
    let d = day % 100;
    (y, m, d)
}

pub fn from_ymd(y: i32, m: u32, d: u32) -> u32 {
    (y as u32) * 10_000 + m * 100 + d
}

/// 距离 1970-01-01 的天数。算法来自 Howard Hinnant 的 civil_from_days。
pub fn days_from_civil(day: u32) -> i64 {
    let (mut y, m, d) = ymd(day);
    y -= (m <= 2) as i32;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp as u64 + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

pub fn civil_from_days(mut z: i64) -> u32 {
    z += 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    from_ymd(y as i32, m as u32, d as u32)
}

pub fn add_days(day: u32, delta: i32) -> u32 {
    civil_from_days(days_from_civil(day) + delta as i64)
}

/// 0 是周一，6 是周日。1970-01-01 是周四。
pub fn weekday_mon(day: u32) -> u8 {
    let z = days_from_civil(day);
    (z + 3).rem_euclid(7) as u8
}

pub fn minute_label(minute: i32) -> String {
    if minute < 0 {
        return "还没有".to_string();
    }
    format!("{:02}:{:02}", minute / 60, minute % 60)
}

pub fn duration_label(ms: u64) -> String {
    let total = ms / 1000;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    if h > 0 {
        format!("{h} 小时 {m} 分")
    } else if m > 0 {
        format!("{m} 分 {} 秒", total % 60)
    } else {
        format!("{} 秒", total % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_and_roundtrip() {
        assert_eq!(days_from_civil(19700101), 0);
        assert_eq!(civil_from_days(0), 19700101);
        assert_eq!(add_days(20261031, 1), 20261101);
        assert_eq!(add_days(20261231, 1), 20270101);
        assert_eq!(add_days(20260301, -1), 20260228);
    }

    #[test]
    fn known_weekday() {
        // 1970-01-01 是周四。周一为 0 时，周四是 3。
        assert_eq!(weekday_mon(19700101), 3);
        // 2026-10-31 是周六。
        assert_eq!(weekday_mon(20261031), 5);
    }
}
