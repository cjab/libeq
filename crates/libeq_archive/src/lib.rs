//! # An Everquest archive file extractor
//! This has only been tested on .s3d files and implements only the bare minimum of functionality.
//! CRC checks for example are completely ignored.
//!
// # Examples
// ```rust
// let archive = eq_archive::read("fixtures/gfaydark.s3d").unwrap();
//
// // List all files in the archive
// let filenames = archive.filenames();
//
// // Iterate over files in the archive
// for (name, data) in archive.files() {
//
// }
//
// ```
//

mod parser;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::ops::ControlFlow;

use nom::error::ErrorKind;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

pub use parser::{Archive, Block, Directory, Footer, Header, IndexEntry};

const UNCOMPRESSED_BLOCK_SIZE: usize = 8192;

#[derive(Default, Debug)]
pub struct EqArchive {
    files: Vec<(String, Vec<u8>)>,
}

impl EqArchive {
    pub fn new() -> Self {
        EqArchive::default()
    }

    pub fn read(filename: &str) -> Result<EqArchive, Error> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let mut archive = Archive::parse(&buffer[..])?.1;

        archive.index_entries.sort_by_key(|e| e.data_offset);

        let files = archive
            .filenames()
            .iter()
            .map(|filename| {
                Ok((
                    filename.to_owned(),
                    archive
                        .get(filename)
                        .ok_or(Error::FileNotFound(filename.to_string()))?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(EqArchive { files })
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, Vec<u8>)> {
        self.files.iter()
    }

    pub fn push(&mut self, filename: &str, data: &[u8]) {
        self.files.push((filename.to_string(), data.to_vec()));
    }

    pub fn remove(&mut self, filename: &str) -> Option<(String, Vec<u8>)> {
        self.files
            .iter()
            .enumerate()
            .find_map(|(idx, (f, _))| if filename == f { Some(idx) } else { None })
            .map(|entry| self.files.remove(entry))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        //let mut files: Vec<_> = self.files;
        //self.files.sort_by_key(|e| filename_crc(&e.0));

        let directory_data = Directory {
            filenames: self
                .files
                .iter()
                .map(|(filename, _)| filename.clone())
                .collect(),
        }
        .to_bytes();

        let entries: Vec<_> = self
            .files
            .iter()
            .map(|(filename, data)| (Some(filename), data))
            .chain([(None, &directory_data)])
            .scan(0, |position, (filename, data)| {
                let blocks: Vec<_> = data
                    .chunks(UNCOMPRESSED_BLOCK_SIZE)
                    .map(|uncompressed_data| {
                        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                        encoder
                            .write_all(uncompressed_data)
                            .expect("Could not compress data");
                        let compressed_data = encoder.finish().expect("Could not compress data");

                        Block {
                            uncompressed_size: uncompressed_data.len() as u32,
                            compressed_data,
                        }
                    })
                    .collect();

                let index_entry = IndexEntry {
                    filename_crc: match filename {
                        Some(f) => filename_crc(f),
                        None => 0xffffffff,
                    },
                    data_offset: *position,
                    uncompressed_size: data.len() as u32,
                };

                *position += blocks.iter().map(|b| b.size()).sum::<usize>() as u32;

                Some((filename, index_entry, blocks))
            })
            .collect();

        let compressed_data_bytes: Vec<_> = entries
            .iter()
            .flat_map(|(_, _, blocks)| blocks.iter().map(|b| b.to_bytes()))
            .flatten()
            .collect();

        let entry_count_bytes = (entries.len() as u32).to_le_bytes();

        let index_bytes: Vec<_> = entries
            .iter()
            .flat_map(|(_, index_entry, _)| index_entry.to_bytes())
            .collect();

        let header_bytes = Header {
            index_offset: (compressed_data_bytes.len() as u32) + Header::SIZE as u32,
            magic_number: Header::MAGIC_NUMBER,
            version: Header::VERSION,
        }
        .to_bytes();

        let footer_bytes = Footer {
            footer_string: Footer::FOOTER_STRING.to_vec(),
            timestamp: 0,
        }
        .to_bytes();

        let bytes = [
            &header_bytes[..],
            &compressed_data_bytes,
            &entry_count_bytes,
            &index_bytes,
            &footer_bytes,
        ]
        .concat();

        Ok(bytes)
    }
}

impl Archive {
    fn filenames(&self) -> Vec<String> {
        let directory_index_entry = self
            .index_entries
            .iter()
            .max_by_key(|e| e.data_offset)
            .expect("Directory entry does not exist");
        let directory_data = directory_index_entry.decompress(&self.blocks);

        let (_, directory) =
            Directory::parse(&directory_data).expect("Failed to parse directory block");
        directory.filenames
    }

    fn get(&self, filename: &str) -> Option<Vec<u8>> {
        self.filenames()
            .iter()
            .position(|f| f.eq_ignore_ascii_case(filename))
            .and_then(|position| {
                self.index_entries
                    .get(position)
                    .map(|entry| entry.decompress(&self.blocks))
            })
    }

    fn files(self) -> impl Iterator<Item = (String, IndexEntry)> {
        self.filenames()
            .into_iter()
            .zip(self.index_entries.into_iter().map(|entry| entry))
    }
}

impl IndexEntry {
    /// Decompress the compresed data blocks belonging to this file and
    /// return the uncompressed data.
    fn decompress(&self, all_blocks: &BTreeMap<usize, Block>) -> Vec<u8> {
        self.get_blocks(all_blocks)
            .iter()
            .flat_map(|block| {
                let mut buf = Vec::new();
                ZlibDecoder::new(&block.compressed_data[..])
                    .read_to_end(&mut buf)
                    .expect("Failed to decompress block");
                buf
            })
            .collect()
    }

    /// Get a range of blocks from the list of all blocks in an archive.
    /// These blocks will contain the data for the file corresponding to this
    /// `IndexEntry`.
    fn get_blocks<'a>(&self, all_blocks: &'a BTreeMap<usize, Block>) -> Vec<&'a Block> {
        // The starting block is found using the `data_offset` field in the `IndexEntry`.
        // The end block depends on the total uncompressed size of all of the
        // blocks gathered. Once the sum of the uncompressed sizes of all blocks matches
        // the `uncompressed_size` field in the `IndexEntry` all blocks belonging
        // to this file have been found.
        let result = all_blocks.range(self.data_offset as usize..).try_fold(
            (0, Vec::new()),
            |(bytes_collected, mut acc), (_, block)| {
                let next_bytes_collected = bytes_collected + block.uncompressed_size;

                if next_bytes_collected == self.uncompressed_size {
                    // Found the last block!
                    acc.push(block);
                    ControlFlow::Break((next_bytes_collected, acc))
                } else if next_bytes_collected < self.uncompressed_size {
                    // Keep looking for more blocks!
                    acc.push(block);
                    ControlFlow::Continue((next_bytes_collected, acc))
                } else {
                    // TODO: Should this function return a Result?
                    //       Ending up here is a pretty good indication the file
                    //       is in some way incorrect or corrupt.
                    panic!("Oh no, your file may be corrupt :S");
                }
            },
        );

        match result {
            ControlFlow::Break((_, blocks)) => blocks,
            ControlFlow::Continue((_, _)) => {
                panic!("Oh no, your file may be corrupt :S. You're short a few blocks!")
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Parser,
    FileNotFound(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<nom::Err<(&[u8], ErrorKind)>> for Error {
    fn from(_: nom::Err<(&[u8], ErrorKind)>) -> Self {
        Self::Parser
    }
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(_: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::Parser
    }
}

fn filename_crc(filename: &str) -> u32 {
    filename
        .bytes()
        .chain(vec![0u8].into_iter()) // Add null string terminator back in
        .fold(0, |crc, byte| {
            let idx = ((crc >> 24) ^ (byte as u32)) & 0xff;
            (crc << 8) ^ CRC_TABLE[idx as usize]
        })
}

const CRC_TABLE: [u32; 256] = build_crc_table();
const fn build_crc_table() -> [u32; 256] {
    const TABLE_SIZE: usize = 256;
    let mut crc_table: [u32; 256] = [0; TABLE_SIZE];

    let mut idx = 0;
    while idx < TABLE_SIZE {
        let mut crc: u32 = (idx as u32) << 24;

        let mut round = 0;
        while round < 8 {
            crc = if (crc & 0x80000000) != 0 {
                (crc << 1) ^ 0x04c11db7
            } else {
                crc << 1
            };
            round += 1;
        }

        crc_table[idx] = crc;
        idx += 1;
    }
    crc_table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut fixture = File::open("fixtures/gfaydark.s3d").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, archive) = Archive::parse(&fixture_data).unwrap();

        let filenames = archive.filenames();

        assert_eq!(filenames[0], "palette.bmp");
    }

    #[test]
    fn read_archive() {
        let original = EqArchive::read("fixtures/gfaydark.s3d").unwrap();

        let mut file = File::create("out.s3d").unwrap();
        file.write_all(&original.to_bytes().unwrap()[..]).unwrap();

        let loaded = EqArchive::read("out.s3d").unwrap();

        let original_filenames: Vec<_> = original
            .files
            .iter()
            .map(|(filename, _)| filename)
            .collect();
        let loaded_filenames: Vec<_> = loaded.files.iter().map(|(filename, _)| filename).collect();
        assert_eq!(original_filenames, loaded_filenames);
    }

    #[test]
    fn modify_archive() {
        let mut archive = EqArchive::new();
        archive.push("test0.bmp", &vec![]);
        archive.push("test1.bmp", &vec![]);
        assert_eq!(
            archive.files,
            vec![
                ("test0.bmp".to_string(), vec![]),
                ("test1.bmp".to_string(), vec![])
            ]
        );
        assert_ne!(archive.to_bytes().unwrap(), vec![]);
        archive.remove("test0.bmp");
        assert_eq!(archive.files, vec![("test1.bmp".to_string(), vec![])]);
        assert_ne!(archive.to_bytes().unwrap(), vec![]);
    }
}
