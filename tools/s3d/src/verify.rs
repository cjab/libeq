use std::io;

use crate::open_archive;

/// Verify all files in the given archives. Returns true if all passed.
pub fn run(files: &[String], verbose: bool) -> bool {
    let mut all_ok = true;

    for (i, path) in files.iter().enumerate() {
        if i > 0 && verbose {
            println!();
        }

        let Some((mut reader, filenames)) = open_archive(path) else {
            all_ok = false;
            continue;
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
        } else {
            println!("{}: {}/{} files OK, {} failed", path, passed, total, failed);
            all_ok = false;
        }
    }

    all_ok
}
