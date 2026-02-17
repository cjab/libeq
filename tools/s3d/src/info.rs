use std::fs;

use crate::fmt::{format_number, format_size, format_timestamp};
use crate::open_archive;

pub fn run(files: &[String], human: bool) {
    for (i, path) in files.iter().enumerate() {
        if i > 0 {
            println!();
        }

        let file_size = match fs::metadata(path) {
            Ok(m) => m.len(),
            Err(e) => {
                eprintln!("{}: {}", path, e);
                continue;
            }
        };

        let Some((mut reader, _filenames)) = open_archive(path) else {
            continue;
        };

        let info = match reader.archive_info() {
            Ok(info) => info,
            Err(e) => {
                eprintln!("{}: {}", path, e);
                continue;
            }
        };

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
    }
}
