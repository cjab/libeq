use std::io::{self, Cursor};

use libeq_archive::EqArchiveReader;

use crate::open_archive;

/// Verify all files in the given archives. Returns true if all passed.
pub fn run(files: &[String], verbose: bool, round_trip: bool) -> bool {
    let mut all_ok = true;

    for (i, path) in files.iter().enumerate() {
        if i > 0 && verbose {
            println!();
        }

        if !verify_read(path, verbose) {
            all_ok = false;
        }

        if round_trip && !verify_round_trip(path) {
            all_ok = false;
        }
    }

    all_ok
}

fn verify_read(path: &str, verbose: bool) -> bool {
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

    let total = passed + failed;
    if failed == 0 {
        println!("{}: {} files OK", path, passed);
        true
    } else {
        println!("{}: {}/{} files OK, {} failed", path, passed, total, failed);
        false
    }
}

fn verify_round_trip(path: &str) -> bool {
    let original = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("{}: {}", path, e);
            return false;
        }
    };

    let mut reader = match EqArchiveReader::read(Cursor::new(&original)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: round-trip: failed to parse: {}", path, e);
            return false;
        }
    };

    let writer = match reader.to_writer() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{}: round-trip: failed to convert to writer: {}", path, e);
            return false;
        }
    };

    let roundtripped = match writer.to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("{}: round-trip: failed to serialize: {}", path, e);
            return false;
        }
    };

    if original == roundtripped {
        println!("{}: round-trip OK ({} bytes)", path, original.len());
        true
    } else {
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
        false
    }
}
