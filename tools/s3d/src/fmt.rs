use jiff::Timestamp;
use jiff::tz::TimeZone;
use libeq_archive::FileInfo;

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

const DAY_NAMES: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

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

fn ordinal_suffix(day: i8) -> &'static str {
    let d = day as u8;
    match (d % 10, d % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}

/// Returns the timezone abbreviation (e.g. "PST", "EST") for a zoned datetime,
/// falling back to a UTC offset string (e.g. "UTC-8") if unavailable.
fn tz_abbrev(ts: &Timestamp, tz: &TimeZone) -> String {
    let zdt = ts.to_zoned(tz.clone());
    jiff::fmt::strtime::format("%Z", &zdt).unwrap_or_else(|_| format_offset(zdt.offset()))
}

fn format_offset(offset: jiff::tz::Offset) -> String {
    let total_seconds = offset.seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds.abs() % 3600) / 60;
    if minutes == 0 {
        format!("UTC{:+}", hours)
    } else {
        format!("UTC{:+}:{:02}", hours, minutes)
    }
}

/// Formats a timestamp as an aligned human-readable date/time string.
/// Weekday is padded to 9 chars (length of "Wednesday") and month to 9 chars
/// (length of "September") so multiple lines align when stacked.
fn format_human_datetime(ts: &Timestamp, tz: &TimeZone) -> String {
    let zdt = ts.to_zoned(tz.clone());
    let dt = zdt.datetime();
    let weekday = DAY_NAMES[zdt.weekday().to_monday_zero_offset() as usize];
    let month = MONTH_NAMES[dt.month() as usize - 1];
    let day = dt.day();
    let suffix = ordinal_suffix(day);
    let year = dt.year();
    let hour = dt.hour();
    let minute = dt.minute();
    let (hour12, ampm) = match hour {
        0 => (12, "AM"),
        1..=11 => (hour, "AM"),
        12 => (12, "PM"),
        _ => (hour - 12, "PM"),
    };
    let abbrev = tz_abbrev(ts, tz);

    format!(
        "{:<9}  {:<9} {}{}, {} {:>2}:{:02} {} {}",
        weekday, month, day, suffix, year, hour12, minute, ampm, abbrev
    )
}

fn format_compact_datetime(ts: &Timestamp, tz: &TimeZone) -> String {
    let zdt = ts.to_zoned(tz.clone());
    let dt = zdt.datetime();
    let abbrev = tz_abbrev(ts, tz);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        abbrev
    )
}

pub(crate) fn format_timestamp(ts: u32, human: bool) -> String {
    let Ok(timestamp) = Timestamp::from_second(ts as i64) else {
        return ts.to_string();
    };

    let local_tz = TimeZone::system();

    if human {
        // Line 1: user's local time
        // Line 2: UTC (indented to align with value column)
        // Line 3: San Diego time with historical note (indented)
        let local = format_human_datetime(&timestamp, &local_tz);
        let utc = format_human_datetime(&timestamp, &TimeZone::UTC);
        let san_diego_tz = TimeZone::get("America/Los_Angeles").unwrap_or_else(|_| TimeZone::UTC);
        let san_diego = format_human_datetime(&timestamp, &san_diego_tz);

        format!(
            "{} (local time)\n                 {}\n                 {} — Verant Interactive, San Diego, CA",
            local, utc, san_diego
        )
    } else {
        // Default: raw value + user's local time compact
        let formatted = format_compact_datetime(&timestamp, &local_tz);
        format!("{} ({})", ts, formatted)
    }
}
