use std::error::Error;

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

pub fn run(files: &[String], verbosity: u8, human: bool) -> Result<(), Box<dyn Error>> {
    let mut failed = Vec::new();

    for (i, path) in files.iter().enumerate() {
        if i > 0 {
            println!();
        }

        let result = match verbosity {
            0 => list(path, files.len() > 1),
            1 => list_verbose(path, files.len() > 1, human),
            _ => list_very_verbose(path, files.len() > 1, human),
        };

        if let Err(e) = result {
            eprintln!("{}", e);
            failed.push(path.as_str());
        }
    }

    if !failed.is_empty() {
        return Err(format!("{} archive(s) failed", failed.len()).into());
    }

    Ok(())
}

fn list(path: &str, show_header: bool) -> Result<(), Box<dyn Error>> {
    let (_reader, filenames) = open_archive(path)?;

    if show_header {
        println!("{}:", path);
    }
    for name in &filenames {
        println!("{}", name);
    }

    Ok(())
}

fn list_verbose(path: &str, show_header: bool, human: bool) -> Result<(), Box<dyn Error>> {
    let (mut reader, filenames) = open_archive(path)?;

    if show_header {
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
        let info = reader
            .info(name)
            .map_err(|e| format!("{}: {}: {}", path, name, e))?
            .ok_or_else(|| format!("{}: {}: not found in index", path, name))?;

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

    Ok(())
}

fn list_very_verbose(path: &str, show_header: bool, human: bool) -> Result<(), Box<dyn Error>> {
    let (mut reader, filenames) = open_archive(path)?;

    if show_header {
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
        let info = reader
            .info(name)
            .map_err(|e| format!("{}: {}: {}", path, name, e))?
            .ok_or_else(|| format!("{}: {}: not found in index", path, name))?;

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

    let dir_info = reader
        .directory_info()
        .map_err(|e| format!("{}: [directory]: {}", path, e))?;

    let ratio = format_ratio(&dir_info);
    println!(
        "{:>10}  {:>10}  {:>12}  {:>6}  {:>5}  [directory]",
        format_number(dir_info.data_offset as u64, human),
        format_size(dir_info.compressed_size as u64, human),
        format_size(dir_info.uncompressed_size as u64, human),
        dir_info.block_count,
        ratio
    );
    total_compressed += dir_info.compressed_size as u64;
    total_uncompressed += dir_info.uncompressed_size as u64;

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

    Ok(())
}
