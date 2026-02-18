use std::io::{self, Write};

use crate::open_archive;

const HELP: &str = "\
s3d get — Extract single file to stdout

Usage: s3d get <archive> <filename>

Writes the raw contents of a single file to stdout.
Errors are printed to stderr.";

pub fn print_help() {
    println!("{}", HELP);
}

pub fn eprint_help() {
    eprintln!("{}", HELP);
}

pub fn run(archive: &str, filename: &str) -> bool {
    let Some((mut reader, _filenames)) = open_archive(archive) else {
        return false;
    };

    let data = match reader.get(filename) {
        Ok(Some(data)) => data,
        Ok(None) => {
            eprintln!("{}: {}: not found in archive", archive, filename);
            return false;
        }
        Err(e) => {
            eprintln!("{}: {}: {}", archive, filename, e);
            return false;
        }
    };

    if let Err(e) = io::stdout().write_all(&data) {
        eprintln!("{}", e);
        return false;
    }

    true
}
