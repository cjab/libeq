use std::error::Error;
use std::io::{self, Cursor};

use libeq_archive::EqArchiveReader;

use crate::open_archive;

const HELP: &str = "\
s3d verify — Verify archive integrity

Usage: s3d verify [options] <archive>...

Reads every file entry and performs a bitwise round-trip check.

Options:
  -v, --verbose          Show per-file results
  -h, --help             Show this help

Aliases: v";

pub fn print_help() {
    println!("{}", HELP);
}

pub fn eprint_help() {
    eprintln!("{}", HELP);
}

/// Verify all files in the given archives.
pub fn run(files: &[String], verbose: bool) -> Result<(), Box<dyn Error>> {
    let mut failed_archives = Vec::new();

    for (i, path) in files.iter().enumerate() {
        if i > 0 && verbose {
            println!();
        }

        if let Err(e) = verify_archive(path, verbose) {
            eprintln!("{}", e);
            failed_archives.push(path.as_str());
        }
    }

    if !failed_archives.is_empty() {
        return Err(format!("{} archive(s) failed verification", failed_archives.len()).into());
    }

    Ok(())
}

fn verify_archive(path: &str, verbose: bool) -> Result<(), Box<dyn Error>> {
    // Phase 1: Read check — decompress every file, verify sizes
    let (mut reader, filenames) = open_archive(path)?;

    let mut passed: usize = 0;
    let mut failed: usize = 0;

    for name in &filenames {
        let expected_size = match reader.info(name) {
            Ok(Some(info)) => info.uncompressed_size as u64,
            Ok(None) => {
                eprintln!("{}: {}: not found in index", path, name);
                failed += 1;
                continue;
            }
            Err(e) => {
                eprintln!("{}: {}: {}", path, name, e);
                failed += 1;
                continue;
            }
        };

        let mut file_reader = match reader.get_reader(name) {
            Ok(Some(r)) => r,
            Ok(None) => {
                eprintln!("{}: {}: not found in index", path, name);
                failed += 1;
                continue;
            }
            Err(e) => {
                eprintln!("{}: {}: {}", path, name, e);
                failed += 1;
                continue;
            }
        };

        let actual_size = match io::copy(&mut file_reader, &mut io::sink()) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("{}: {}: decompression failed: {}", path, name, e);
                if verbose {
                    println!("{}: FAILED", name);
                }
                failed += 1;
                continue;
            }
        };

        if actual_size != expected_size {
            eprintln!(
                "{}: {}: size mismatch: expected {}, got {}",
                path, name, expected_size, actual_size
            );
            if verbose {
                println!("{}: FAILED", name);
            }
            failed += 1;
            continue;
        }

        if verbose {
            println!("{}: OK", name);
        }
        passed += 1;
    }

    if failed > 0 {
        let total = passed + failed;
        return Err(format!("{}: {}/{} files OK, {} failed", path, passed, total, failed).into());
    }

    // Phase 2: Round-trip check — read entire archive, serialize back, compare bytes
    let original = std::fs::read(path).map_err(|e| format!("{}: {}", path, e))?;

    let mut rt_reader = EqArchiveReader::open(Cursor::new(&original))
        .map_err(|e| format!("{}: round-trip: failed to parse: {}", path, e))?;

    let writer = rt_reader
        .to_writer(Cursor::new(Vec::new()))
        .map_err(|e| format!("{}: round-trip: failed to convert to writer: {}", path, e))?;

    let roundtripped = writer
        .finish()
        .map_err(|e| format!("{}: round-trip: failed to serialize: {}", path, e))?
        .into_inner();

    if original != roundtripped {
        let first_diff = original
            .iter()
            .zip(roundtripped.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(std::cmp::min(original.len(), roundtripped.len()));

        return Err(format!(
            "{}: round-trip FAILED (original {} bytes, roundtripped {} bytes, first difference at byte {})",
            path,
            original.len(),
            roundtripped.len(),
            first_diff
        ).into());
    }

    println!(
        "{}: {} files OK, round-trip OK ({} bytes)",
        path,
        passed,
        original.len()
    );

    Ok(())
}
