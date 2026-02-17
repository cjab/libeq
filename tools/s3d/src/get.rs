use std::io::{self, Write};

use crate::open_archive;

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
