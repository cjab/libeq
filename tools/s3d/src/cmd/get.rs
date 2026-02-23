use std::error::Error;
use std::io;

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

pub fn run(archive: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    let (mut reader, _) = open_archive(archive)?;

    let mut file_reader = reader
        .get_reader(filename)
        .map_err(|e| format!("{}: {}: {}", archive, filename, e))?
        .ok_or_else(|| format!("{}: {}: not found in archive", archive, filename))?;

    io::copy(&mut file_reader, &mut io::stdout())?;

    Ok(())
}
