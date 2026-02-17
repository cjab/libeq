use libeq_archive::FileInfo;

use crate::open_archive;

pub fn run(files: &[String], verbosity: u8, human: bool) {
    match verbosity {
        0 => list(files),
        1 => list_verbose(files, human),
        _ => list_very_verbose(files, human),
    }
}

fn list(files: &[String]) {
    for (i, path) in files.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let Some((_reader, filenames)) = open_archive(path) else {
            continue;
        };

        if files.len() > 1 {
            println!("{}:", path);
        }
        for name in &filenames {
            println!("{}", name);
        }
    }
}

fn list_verbose(files: &[String], human: bool) {
    for (i, path) in files.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let Some((mut reader, filenames)) = open_archive(path) else {
            continue;
        };

        if files.len() > 1 {
            println!("{}:", path);
        }

        println!(
            "{:>10}  {:>12}  {:>5}  name",
            "compressed", "uncompressed", "ratio"
        );

        let mut total_compressed: u64 = 0;
        let mut total_uncompressed: u64 = 0;
        let mut file_count: usize = 0;

        for name in &filenames {
            let info = match reader.info(name) {
                Ok(Some(info)) => info,
                Ok(None) => {
                    eprintln!("{}: {}: not found in index", path, name);
                    continue;
                }
                Err(e) => {
                    eprintln!("{}: {}: {}", path, name, e);
                    continue;
                }
            };

            let ratio = format_ratio(&info);
            println!(
                "{:>10}  {:>12}  {:>5}  {}",
                format_size(info.compressed_size as u64, human),
                format_size(info.uncompressed_size as u64, human),
                ratio,
                name
            );

            total_compressed += info.compressed_size as u64;
            total_uncompressed += info.uncompressed_size as u64;
            file_count += 1;
        }

        let total_ratio = format_total_ratio(total_compressed, total_uncompressed);

        println!(
            "{:>10}  {:>12}  {:>5}",
            "----------", "------------", "-----"
        );
        println!(
            "{:>10}  {:>12}  {:>5}  {} files",
            format_size(total_compressed, human),
            format_size(total_uncompressed, human),
            total_ratio,
            file_count
        );
    }
}

fn list_very_verbose(files: &[String], human: bool) {
    for (i, path) in files.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let Some((mut reader, filenames)) = open_archive(path) else {
            continue;
        };

        if files.len() > 1 {
            println!("{}:", path);
        }

        println!(
            "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}  name",
            "offset", "compressed", "uncompressed", "blocks", "ratio"
        );

        let mut total_compressed: u64 = 0;
        let mut total_uncompressed: u64 = 0;
        let mut file_count: usize = 0;

        for name in &filenames {
            let info = match reader.info(name) {
                Ok(Some(info)) => info,
                Ok(None) => {
                    eprintln!("{}: {}: not found in index", path, name);
                    continue;
                }
                Err(e) => {
                    eprintln!("{}: {}: {}", path, name, e);
                    continue;
                }
            };

            let ratio = format_ratio(&info);
            println!(
                "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}  {}",
                format_offset(info.data_offset as u64, human),
                format_size(info.compressed_size as u64, human),
                format_size(info.uncompressed_size as u64, human),
                info.block_count,
                ratio,
                name
            );

            total_compressed += info.compressed_size as u64;
            total_uncompressed += info.uncompressed_size as u64;
            file_count += 1;
        }

        let total_ratio = format_total_ratio(total_compressed, total_uncompressed);

        println!(
            "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}",
            "", "----------", "------------", "", "-----"
        );
        println!(
            "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}  {} files",
            "",
            format_size(total_compressed, human),
            format_size(total_uncompressed, human),
            "",
            total_ratio,
            file_count
        );
    }
}

fn format_ratio(info: &FileInfo) -> String {
    if info.uncompressed_size > 0 {
        format!(
            "{:.1}%",
            info.compressed_size as f64 / info.uncompressed_size as f64 * 100.0
        )
    } else {
        "0.0%".to_string()
    }
}

fn format_total_ratio(compressed: u64, uncompressed: u64) -> String {
    if uncompressed > 0 {
        format!("{:.1}%", compressed as f64 / uncompressed as f64 * 100.0)
    } else {
        "0.0%".to_string()
    }
}

fn format_size(bytes: u64, human: bool) -> String {
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

fn format_offset(offset: u64, human: bool) -> String {
    if !human {
        return offset.to_string();
    }
    let s = offset.to_string();
    if s.len() <= 3 {
        return s;
    }
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}
