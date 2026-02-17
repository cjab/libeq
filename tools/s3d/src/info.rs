use std::fs;

use crate::open_archive;

pub fn run(files: &[String]) {
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
        println!("  size:          {}", file_size);
        println!("  format:        PFS (version {:#010x})", info.version);
        println!("  files:         {}", info.file_count);
        println!("  index offset:  {}", info.index_offset);

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
            println!("  timestamp:     {} (unix)", ts);
        }
    }
}
