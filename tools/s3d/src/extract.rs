use std::fs;
use std::path::Path;

use crate::open_archive;

const HELP: &str = "\
s3d extract — Extract files from archive

Usage: s3d extract [options] <archive> [files...]

Extracts files from an archive. If no filenames are given,
all files are extracted.

Options:
  -o, --output <dir>     Output directory (created if needed)
  -v, --verbose          Print filenames as extracted
  -h, --help             Show this help

Aliases: x";

pub fn print_help() {
    println!("{}", HELP);
}

pub fn eprint_help() {
    eprintln!("{}", HELP);
}

pub fn run(archive: &str, files: &[String], output: Option<&str>, verbose: bool) -> bool {
    let Some((mut reader, all_filenames)) = open_archive(archive) else {
        return false;
    };

    let to_extract: Vec<&str> = if files.is_empty() {
        all_filenames.iter().map(|s| s.as_str()).collect()
    } else {
        files.iter().map(|s| s.as_str()).collect()
    };

    if let Some(dir) = output
        && let Err(e) = fs::create_dir_all(dir)
    {
        eprintln!("{}: {}", dir, e);
        return false;
    }

    let mut all_ok = true;

    for name in &to_extract {
        let data = match reader.get(name) {
            Ok(Some(data)) => data,
            Ok(None) => {
                eprintln!("{}: {}: not found in archive", archive, name);
                all_ok = false;
                continue;
            }
            Err(e) => {
                eprintln!("{}: {}: {}", archive, name, e);
                all_ok = false;
                continue;
            }
        };

        let out_path = match output {
            Some(dir) => Path::new(dir).join(name),
            None => Path::new(name).to_path_buf(),
        };

        if let Err(e) = fs::write(&out_path, &data) {
            eprintln!("{}: {}", out_path.display(), e);
            all_ok = false;
            continue;
        }

        if verbose {
            println!("{}", name);
        }
    }

    all_ok
}
