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
