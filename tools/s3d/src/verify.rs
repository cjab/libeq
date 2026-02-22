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

/// Verify all files in the given archives. Returns true if all passed.
pub fn run(files: &[String], verbose: bool) -> bool {
    let mut all_ok = true;

    for (i, path) in files.iter().enumerate() {
        if i > 0 && verbose {
            println!();
        }

        if !verify_archive(path, verbose) {
            all_ok = false;
        }
    }

    all_ok
}

fn verify_archive(path: &str, verbose: bool) -> bool {
    // Phase 1: Read check — decompress every file, verify sizes
    let Some((mut reader, filenames)) = open_archive(path) else {
        return false;
    };

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
        eprintln!("{}: {}/{} files OK, {} failed", path, passed, total, failed);
        return false;
    }

    // Phase 2: Round-trip check — read entire archive, serialize back, compare bytes
    let original = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("{}: {}", path, e);
            return false;
        }
    };

    let mut rt_reader = match EqArchiveReader::open(Cursor::new(&original)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: round-trip: failed to parse: {}", path, e);
            return false;
        }
    };

    let writer = match rt_reader.to_writer(Cursor::new(Vec::new())) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{}: round-trip: failed to convert to writer: {}", path, e);
            return false;
        }
    };

    let roundtripped = match writer.finish() {
        Ok(cursor) => cursor.into_inner(),
        Err(e) => {
            eprintln!("{}: round-trip: failed to serialize: {}", path, e);
            return false;
        }
    };

    if original != roundtripped {
        let first_diff = original
            .iter()
            .zip(roundtripped.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(std::cmp::min(original.len(), roundtripped.len()));

        eprintln!(
            "{}: round-trip FAILED (original {} bytes, roundtripped {} bytes, first difference at byte {})",
            path,
            original.len(),
            roundtripped.len(),
            first_diff
        );
        return false;
    }

    println!(
        "{}: {} files OK, round-trip OK ({} bytes)",
        path,
        passed,
        original.len()
    );
    true
}
