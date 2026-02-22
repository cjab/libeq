use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use libeq_archive::EqArchiveWriter;

const HELP: &str = "\
s3d create — Create archive from files

Usage: s3d create [options] <archive> <files|dirs>...

Creates a new archive from the given files and directories.
Directories are traversed recursively; files are flattened
to basenames.

Options:
  -f, --force            Overwrite existing archive
  -v, --verbose          Print filenames as added
  -h, --help             Show this help

Aliases: c";

pub fn print_help() {
    println!("{}", HELP);
}

pub fn eprint_help() {
    eprintln!("{}", HELP);
}

/// Create a new archive from input files and directories. Returns true if all succeeded.
pub fn run(archive: &str, inputs: &[String], verbose: bool, force: bool) -> bool {
    if !force && Path::new(archive).exists() {
        eprintln!("{}: already exists (use -f to overwrite)", archive);
        return false;
    }

    let mut files = Vec::new();
    for input in inputs {
        let path = Path::new(input);
        match collect_files(path) {
            Ok(collected) => {
                if collected.is_empty() {
                    eprintln!("{}: no files found", input);
                }
                files.extend(collected);
            }
            Err(e) => {
                eprintln!("{}: {}", input, e);
                return false;
            }
        }
    }

    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    // Detect duplicate basenames — last one wins, warn about earlier ones
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    for path in &files {
        let basename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };
        if let Some(prev) = seen.insert(basename.clone(), path.clone()) {
            eprintln!(
                "warning: duplicate filename '{}', using {} (overwriting {})",
                basename,
                path.display(),
                prev.display()
            );
        }
    }

    let file = match fs::File::create(archive) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {}", archive, e);
            return false;
        }
    };

    let mut writer = match EqArchiveWriter::create(file) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{}: failed to create archive: {}", archive, e);
            return false;
        }
    };

    let mut all_ok = true;

    for path in &files {
        let basename = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: {}", path.display(), e);
                all_ok = false;
                continue;
            }
        };

        if let Err(e) = writer.insert(&basename, file) {
            eprintln!("{}: failed to add to archive: {}", basename, e);
            all_ok = false;
            continue;
        }

        if verbose {
            println!("{}", basename);
        }
    }

    if let Err(e) = writer.finish() {
        eprintln!("{}: failed to finish archive: {}", archive, e);
        return false;
    }

    all_ok
}

fn collect_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let child = entry?.path();
            files.extend(collect_files(&child)?);
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}: not a file or directory", path.display()),
        ));
    }
    Ok(files)
}
