use std::error::Error;
use std::fs;

use crate::fmt::{format_number, format_size, format_timestamp};
use crate::open_archive;

const HELP: &str = "\
s3d info — Display archive metadata

Usage: s3d info [options] <archive>...

Options:
  -r, --raw              Show raw numeric values
  -h, --help             Show this help

Aliases: i";

pub fn print_help() {
    println!("{}", HELP);
}

pub fn eprint_help() {
    eprintln!("{}", HELP);
}

pub fn run(files: &[String], human: bool) -> Result<(), Box<dyn Error>> {
    let mut failed = Vec::new();

    for (i, path) in files.iter().enumerate() {
        if i > 0 {
            println!();
        }

        if let Err(e) = show_info(path, human) {
            eprintln!("{}", e);
            failed.push(path.as_str());
        }
    }

    if !failed.is_empty() {
        return Err(format!("{} archive(s) failed", failed.len()).into());
    }

    Ok(())
}

fn show_info(path: &str, human: bool) -> Result<(), Box<dyn Error>> {
    let file_size = fs::metadata(path)
        .map_err(|e| format!("{}: {}", path, e))?
        .len();

    let (mut reader, _filenames) = open_archive(path)?;

    let info = reader
        .archive_info()
        .map_err(|e| format!("{}: {}", path, e))?;

    println!("{}:", path);
    println!("  size:          {}", format_size(file_size, human));
    println!("  format:        PFS (version {:#010x})", info.version);
    println!(
        "  files:         {}",
        format_number(info.file_count as u64, human)
    );
    println!(
        "  index offset:  {}",
        format_number(info.index_offset as u64, human)
    );

    match info.footer_string {
        Some(s) => {
            let footer = String::from_utf8_lossy(&s);
            println!("  footer:        {}", footer);
        }
        None => {
            println!("  footer:        none");
        }
    }

    if let Some(ts) = info.timestamp {
        println!("  timestamp:     {}", format_timestamp(ts, human));
    }

    Ok(())
}
