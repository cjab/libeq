use std::collections::HashSet;
use std::io::{Read, Seek, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::crc::FilenameCrc;
use crate::error::Error;
use crate::parser::{Block, BlockHeader, Directory, Footer, Header, IndexEntry};

pub struct EqArchiveWriter<W> {
    writer: W,
    entries: Vec<(String, IndexEntry)>,
    removed: HashSet<String>,
    directory: Option<IndexEntry>,
    footer: Option<Footer>,
}

impl<W: Read + Write + Seek> EqArchiveWriter<W> {
    pub fn create(mut writer: W) -> Self {
        writer.write_all(&Header::default().to_bytes());
        Self {
            writer,
            entries: Vec::new(),
            removed: HashSet::new(),
            directory: None,
            footer: None,
        }
    }

    pub(crate) fn from_reader(
        existing: Vec<(String, Vec<Block>)>,
        directory: Vec<Block>,
        footer: Option<Footer>,
    ) -> Self {
        EqArchiveWriter {
            existing,
            directory: Some(directory),
            footer,
            ..EqArchiveWriter::default()
        }
    }

    pub fn push(&mut self, filename: impl Into<String>, data: impl Into<Vec<u8>>) {
        let filename = filename.into();
        let data = data.into();
        self.remove(&filename);
        // Clear the directory, we now need to generate a new one
        self.directory = None;
        self.added.retain(|(f, _)| f != &filename);
        self.added.push((filename, data));
    }

    pub fn remove(&mut self, filename: &str) {
        // Clear the directory, we now need to generate a new one
        self.directory = None;
        self.existing.retain(|(f, _)| f != filename);
        self.added.retain(|(f, _)| f != filename);
    }

    pub fn filenames(&self) -> Vec<String> {
        let mut names: Vec<_> = self
            .existing
            .iter()
            .map(|e| &e.0)
            .cloned()
            .chain(self.added.iter().map(|a| &a.0).cloned())
            .collect();
        // Blocks are sorted by filename CRC so the filenames in the directory
        // should be also. It isn't strictly necessary for this implementation
        // in that we lookup based on CRC but other implementations of s3d _DO_
        // require this ordering because lookup happens based on block ordering.
        names.sort_by_key(|f| FilenameCrc::new(f));
        names
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let directory_blocks = match &self.directory {
            Some(d) => d,
            None => &encode(
                &Directory {
                    filenames: self.filenames(),
                }
                .to_bytes(),
            )?,
        };
        let added_entries: Vec<_> = self
            .added
            .iter()
            .map(|(filename, data)| Ok((filename.clone(), encode(data)?)))
            .collect::<Result<_, Error>>()?;

        let file_entries: Vec<_> = self
            // Existing entries
            .existing
            .iter()
            // Followed by new entries
            .chain(added_entries.iter())
            .collect();

        let blocks_start = Header::SIZE;
        let mut index = Vec::new();
        let mut block_bytes = Vec::new();
        for (filename, blocks) in file_entries {
            let data_offset = blocks_start + block_bytes.len() as u32;
            for b in blocks.iter() {
                block_bytes.extend_from_slice(&b.to_bytes());
            }
            index.push(IndexEntry {
                filename_crc: FilenameCrc::new(filename).into(),
                data_offset,
                uncompressed_size: blocks.iter().map(|b| b.uncompressed_size).sum(),
            });
        }

        let directory_offset = blocks_start + block_bytes.len() as u32;
        for block in directory_blocks {
            block_bytes.extend_from_slice(&block.to_bytes());
        }

        index.push(IndexEntry {
            filename_crc: FilenameCrc::DIRECTORY.into(),
            data_offset: directory_offset,
            uncompressed_size: directory_blocks.iter().map(|b| b.uncompressed_size).sum(),
        });
        // Blocks are sorted by filename CRC . It isn't strictly necessary for
        // this implementation in that we lookup based on CRC but other
        // implementations of s3d _DO_ require this ordering because lookup
        // happens based on block ordering. Filenames in the directory match
        // this ordering.
        index.sort_by_key(|i| FilenameCrc::from(i.filename_crc));
        let index_bytes: Vec<_> = index.iter().flat_map(|i| i.to_bytes()).collect();

        let entry_count_bytes = (index.len() as u32).to_le_bytes();
        let header_bytes = Header {
            index_offset: (block_bytes.len() as u32) + Header::SIZE,
            magic_number: Header::MAGIC_NUMBER,
            version: Header::VERSION,
        }
        .to_bytes();

        Ok([
            &header_bytes[..],
            &block_bytes,
            &entry_count_bytes,
            &index_bytes,
            &self.footer_bytes(),
        ]
        .concat())
    }

    fn footer_bytes(&self) -> Vec<u8> {
        match (&self.directory, &self.footer) {
            // Preserve the footer if it exists and no changes have been made
            (Some(_), Some(f)) => f.to_bytes(),
            // Update the timestamp if footer exists and changes have been made
            (None, Some(f)) => Footer {
                footer_string: f.footer_string,
                timestamp: current_unix_timestamp(),
            }
            .to_bytes(),
            // If the original archive had no footer do not add one
            (Some(_), None) => vec![],
            // This is a new archive may as well add a footer
            (None, None) => Footer {
                footer_string: Footer::FOOTER_STRING,
                timestamp: current_unix_timestamp(),
            }
            .to_bytes(),
        }
    }
}

fn current_unix_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

fn encode(data: &[u8]) -> Result<Vec<Block>, Error> {
    let mut blocks = Vec::new();
    for chunk in data.chunks(BlockHeader::MAX_UNCOMPRESSED_SIZE) {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(chunk)?;
        let compressed_data = encoder.finish()?;
        blocks.push(Block {
            uncompressed_size: chunk.len() as u32,
            compressed_data,
        })
    }
    Ok(blocks)
}
