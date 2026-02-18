use crate::fmt::{format_number, format_ratio, format_size, format_total_ratio};
use crate::open_archive;

const HELP: &str = "\
s3d list — List archive contents

Usage: s3d list [options] <archive>...

Options:
  -v, --verbose          Show compressed/uncompressed sizes and ratio
  -vv                    Also show offsets and block counts
  -r, --raw              Show raw numeric values
  -h, --help             Show this help

Aliases: ls";

pub fn print_help() {
    println!("{}", HELP);
}

pub fn eprint_help() {
    eprintln!("{}", HELP);
}

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
                format_number(info.data_offset as u64, human),
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

        match reader.directory_info() {
            Ok(info) => {
                let ratio = format_ratio(&info);
                println!(
                    "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}  [directory]",
                    format_number(info.data_offset as u64, human),
                    format_size(info.compressed_size as u64, human),
                    format_size(info.uncompressed_size as u64, human),
                    info.block_count,
                    ratio
                );
                total_compressed += info.compressed_size as u64;
                total_uncompressed += info.uncompressed_size as u64;
            }
            Err(e) => {
                eprintln!("{}: [directory]: {}", path, e);
            }
        }

        let total_ratio = format_total_ratio(total_compressed, total_uncompressed);

        println!(
            "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}",
            "", "----------", "------------", "", "-----"
        );
        println!(
            "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}  {} files + directory",
            "",
            format_size(total_compressed, human),
            format_size(total_uncompressed, human),
            "",
            total_ratio,
            file_count
        );
    }
}
