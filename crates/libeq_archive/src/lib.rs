//! # An Everquest archive file extractor
//!
//! # Examples
//! ```rust,no_run
//! use std::io::{Cursor, Write};
//! use libeq_archive::EqArchiveReader;
//! use libeq_archive::EqArchiveWriter;
//!
//! let file = std::fs::File::open("fixtures/gfaydark.s3d").unwrap();
//!
//! // Open the archive
//! let mut reader = EqArchiveReader::open(file).unwrap();
//!
//! // List all files in the archive
//! let filenames = reader.filenames().unwrap();
//!
//! // Iterate over files in the archive
//! let files: Vec<_> = filenames.iter().map(|name| {
//!     (name, reader.get(name).unwrap(), reader.info(name).unwrap())
//! }).collect();
//!
//! let new_file = std::fs::File::create("gfaydark-new.s3d").unwrap();
//! let mut writer = reader.to_writer(new_file).unwrap();
//!
//! // Add a new file
//! writer.insert("new-file", Cursor::new(vec![0xde, 0xad, 0xbe, 0xef]));
//!
//! // Finish the new archive
//! writer.finish();
//!
//! ```
//!
mod crc;
mod error;
mod parser;
mod read;
mod write;

pub use error::Error;
pub use read::ArchiveInfo;
pub use read::EqArchiveReader;
pub use read::EqFileReader;
pub use read::FileInfo;
pub use write::EqArchiveWriter;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn crud() {
        let test_files = [
            ("test-file0", vec![0xde, 0xad, 0xbe, 0xef]),
            ("test-file1", vec![0xca, 0xfe, 0xba, 0xbe]),
        ];

        // Create
        let bytes = Vec::new();
        let mut writer = EqArchiveWriter::create(Cursor::new(bytes)).unwrap();
        for f in &test_files {
            writer.insert(f.0, Cursor::new(&f.1)).unwrap();
        }
        let filenames = writer.filenames();

        assert_eq!(filenames.len(), test_files.len());
        for t in &test_files {
            assert!(filenames.iter().any(|f| t.0 == f));
        }
        let bytes = writer.finish().unwrap().into_inner();

        // Read
        let mut reader = EqArchiveReader::open(Cursor::new(bytes)).unwrap();

        assert_eq!(filenames.len(), test_files.len());
        for t in &test_files {
            assert!(filenames.iter().any(|f| t.0 == f));
            assert_eq!(&reader.get(t.0).unwrap().unwrap(), &t.1);
        }

        let more_test_files = [("test-file2", vec![0x13u8, 0x37, 0xfe, 0xe7])];

        // Update
        let out = Vec::new();
        let mut editor = reader.to_writer(Cursor::new(out)).unwrap();
        editor
            .insert(more_test_files[0].0, Cursor::new(&more_test_files[0].1))
            .unwrap();
        assert_eq!(editor.filenames().len(), test_files.len() + 1);
        for t in &test_files {
            assert!(editor.filenames().iter().any(|f| t.0 == f));
        }
        for t in &more_test_files {
            assert!(editor.filenames().iter().any(|f| t.0 == f));
        }

        // Delete
        editor.remove(more_test_files[0].0);
        assert_eq!(editor.filenames().len(), test_files.len());
        for t in &test_files {
            assert!(editor.filenames().iter().any(|f| t.0 == f));
        }
    }

    #[test]
    fn info() {
        let test_files = [
            ("test-file0", vec![0xde, 0xad, 0xbe, 0xef]),
            ("test-file1", vec![0xca, 0xfe, 0xba, 0xbe]),
        ];
        let bytes = Vec::new();
        let mut writer = EqArchiveWriter::create(Cursor::new(bytes)).unwrap();
        for f in &test_files {
            writer.insert(f.0, Cursor::new(&f.1)).unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();

        let mut reader = EqArchiveReader::open(Cursor::new(bytes)).unwrap();
        assert_eq!(
            reader.info("test-file0").unwrap().unwrap(),
            FileInfo {
                data_offset: 12,
                compressed_size: 12,
                uncompressed_size: 4,
                block_count: 1
            }
        );
        assert_eq!(
            reader.info("test-file1").unwrap().unwrap(),
            FileInfo {
                data_offset: 32,
                compressed_size: 12,
                uncompressed_size: 4,
                block_count: 1
            }
        );
        assert_eq!(reader.info("missing-file").unwrap(), None);
    }
}
