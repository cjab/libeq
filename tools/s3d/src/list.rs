use std::fs::File;

use libeq_archive::{EqArchiveReader, FileInfo};

fn open_archive(path: &str) -> Option<(EqArchiveReader<File>, Vec<String>)> {
    let file = File::open(path)
        .map_err(|e| eprintln!("{}: {}", path, e))
        .ok()?;
    let mut reader = EqArchiveReader::open(file)
        .map_err(|e| eprintln!("{}: {}", path, e))
        .ok()?;
    let filenames = reader
        .filenames()
        .map_err(|e| eprintln!("{}: {}", path, e))
        .ok()?;
    Some((reader, filenames))
}

pub fn run(files: &[String], verbosity: u8) {
    match verbosity {
        0 => list(files),
        1 => list_verbose(files),
        _ => list_very_verbose(files),
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

fn list_verbose(files: &[String]) {
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
                info.compressed_size, info.uncompressed_size, ratio, name
            );

            total_compressed += info.compressed_size as u64;
            total_uncompressed += info.uncompressed_size as u64;
            file_count += 1;
        }

        let total_ratio = if total_uncompressed > 0 {
            format!(
                "{:.1}%",
                total_compressed as f64 / total_uncompressed as f64 * 100.0
            )
        } else {
            "0.0%".to_string()
        };

        println!(
            "{:>10}  {:>12}  {:>5}",
            "----------", "------------", "-----"
        );
        println!(
            "{:>10}  {:>12}  {:>5}  {} files",
            total_compressed, total_uncompressed, total_ratio, file_count
        );
    }
}

fn list_very_verbose(files: &[String]) {
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
            "{:>8}  {:>10}  {:>12}  {:>6}  {:>5}  name",
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
                "{:>8}  {:>10}  {:>12}  {:>6}  {:>5}  {}",
                info.data_offset,
                info.compressed_size,
                info.uncompressed_size,
                info.block_count,
                ratio,
                name
            );

            total_compressed += info.compressed_size as u64;
            total_uncompressed += info.uncompressed_size as u64;
            file_count += 1;
        }

        let total_ratio = if total_uncompressed > 0 {
            format!(
                "{:.1}%",
                total_compressed as f64 / total_uncompressed as f64 * 100.0
            )
        } else {
            "0.0%".to_string()
        };

        println!(
            "{:>8}  {:>10}  {:>12}  {:>6}  {:>5}",
            "", "----------", "------------", "", "-----"
        );
        println!(
            "{:>8}  {:>10}  {:>12}  {:>6}  {:>5}  {} files",
            "", total_compressed, total_uncompressed, "", total_ratio, file_count
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
