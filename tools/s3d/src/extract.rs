use std::error::Error;
use std::fs::{self, File};
use std::io;
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

pub fn run(
    archive: &str,
    files: &[String],
    output: Option<&str>,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    let (mut reader, all_filenames) = open_archive(archive)?;

    let to_extract: Vec<&str> = if files.is_empty() {
        all_filenames.iter().map(|s| s.as_str()).collect()
    } else {
        files.iter().map(|s| s.as_str()).collect()
    };

    if let Some(dir) = output {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir, e))?;
    }

    for name in &to_extract {
        let mut file_reader = reader
            .get_reader(name)
            .map_err(|e| format!("{}: {}: {}", archive, name, e))?
            .ok_or_else(|| format!("{}: {}: not found in archive", archive, name))?;

        let out_path = match output {
            Some(dir) => Path::new(dir).join(name),
            None => Path::new(name).to_path_buf(),
        };

        let mut out_file =
            File::create(&out_path).map_err(|e| format!("{}: {}", out_path.display(), e))?;
        io::copy(&mut file_reader, &mut out_file)
            .map_err(|e| format!("{}: {}", out_path.display(), e))?;

        if verbose {
            println!("{}", name);
        }
    }

    Ok(())
}
