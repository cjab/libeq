use libeq_archive::FileInfo;
use time_format::{components_utc, strftime_utc};

pub(crate) fn format_ratio(info: &FileInfo) -> String {
    if info.uncompressed_size > 0 {
        format!(
            "{:.1}%",
            info.compressed_size as f64 / info.uncompressed_size as f64 * 100.0
        )
    } else {
        "0.0%".to_string()
    }
}

pub(crate) fn format_total_ratio(compressed: u64, uncompressed: u64) -> String {
    if uncompressed > 0 {
        format!("{:.1}%", compressed as f64 / uncompressed as f64 * 100.0)
    } else {
        "0.0%".to_string()
    }
}

pub(crate) fn format_size(bytes: u64, human: bool) -> String {
    if !human {
        return bytes.to_string();
    }
    const K: f64 = 1024.0;
    const M: f64 = K * 1024.0;
    const G: f64 = M * 1024.0;
    let b = bytes as f64;
    if b >= G {
        format!("{:.1}G", b / G)
    } else if b >= M {
        format!("{:.1}M", b / M)
    } else if b >= K {
        format!("{:.1}K", b / K)
    } else {
        bytes.to_string()
    }
}

pub(crate) fn format_number(n: u64, human: bool) -> String {
    if !human {
        return n.to_string();
    }
    let s = n.to_string();
    if s.len() <= 3 {
        return s;
    }
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn ordinal_suffix(day: u8) -> &'static str {
    match (day % 10, day % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

pub(crate) fn format_timestamp(ts: u32, human: bool) -> String {
    if human {
        let Ok(c) = components_utc(ts as i64) else {
            return ts.to_string();
        };
        let month = MONTH_NAMES[c.month as usize - 1];
        let suffix = ordinal_suffix(c.month_day);
        let (hour, ampm) = match c.hour {
            0 => (12, "AM"),
            1..=11 => (c.hour, "AM"),
            12 => (12, "PM"),
            _ => (c.hour - 12, "PM"),
        };
        format!(
            "{} {}{}, {} {}:{:02} {} UTC",
            month, c.month_day, suffix, c.year, hour, c.min, ampm
        )
    } else {
        let formatted = strftime_utc("%Y-%m-%d %H:%M:%S UTC", ts as i64)
            .unwrap_or_else(|_| "unknown".to_string());
        format!("{} ({})", ts, formatted)
    }
}
