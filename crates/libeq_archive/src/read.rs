use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use flate2::read::ZlibDecoder;

use crate::crc::FilenameCrc;
use crate::error::Error;
use crate::parser::{Block, BlockHeader, Directory, Footer, Header, IndexEntry};
use crate::write::EqArchiveWriter;

// Limits
const MAX_ENTRY_COUNT: u32 = 100_000; // 100k files

#[derive(Debug, Eq, PartialEq)]
pub struct FileInfo {
    pub data_offset: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub block_count: u32,
}

pub struct EqArchiveReader<R> {
    reader: R,
    index: HashMap<FilenameCrc, IndexEntry>,
    directory: IndexEntry,
    footer: Option<Footer>,
}

impl EqArchiveReader<Cursor<Vec<u8>>> {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        from_reader(Cursor::new(buf))
    }
}

impl<R: Read + Seek> EqArchiveReader<R> {
    pub fn open(reader: R) -> Result<Self, Error> {
        from_reader(reader)
    }

    pub fn get(&mut self, filename: &str) -> Result<Option<Vec<u8>>, Error> {
        let Some(mut reader) = self.get_reader(filename)? else {
            return Ok(None);
        };
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(Some(buf))
    }

    pub fn get_reader(&mut self, filename: &str) -> Result<Option<EqFileReader>, Error> {
        let Some(blocks) = self.get_blocks_for(filename)? else {
            return Ok(None);
        };
        Ok(Some(EqFileReader::new(blocks)?))
    }

    pub fn info(&mut self, filename: &str) -> Result<Option<FileInfo>, Error> {
        let crc = FilenameCrc::new(filename);
        let Some(entry) = self.index.get(&crc) else {
            return Ok(None);
        };
        let data_offset = entry.data_offset;
        let uncompressed_size = entry.uncompressed_size;

        let headers = self.read_block_headers(data_offset, uncompressed_size)?;
        Ok(Some(FileInfo {
            data_offset,
            compressed_size: headers.iter().map(|h| h.compressed_size).sum(),
            uncompressed_size,
            block_count: headers.len() as u32,
        }))
    }

    pub fn directory_info(&mut self) -> Result<FileInfo, Error> {
        let data_offset = self.directory.data_offset;
        let uncompressed_size = self.directory.uncompressed_size;
        let headers = self.read_block_headers(data_offset, uncompressed_size)?;
        Ok(FileInfo {
            data_offset,
            compressed_size: headers.iter().map(|h| h.compressed_size).sum(),
            uncompressed_size,
            block_count: headers.len() as u32,
        })
    }

    pub fn filenames(&mut self) -> Result<Vec<String>, Error> {
        let blocks =
            self.read_blocks(self.directory.data_offset, self.directory.uncompressed_size)?;
        let mut data = Vec::with_capacity(self.directory.uncompressed_size as usize);
        let mut reader = EqFileReader::new(blocks)?;
        reader.read_to_end(&mut data)?;
        let mut cursor = Cursor::new(data);
        let directory = Directory::read(&mut cursor)?;
        Ok(directory.filenames)
    }

    pub fn to_writer(&mut self) -> Result<EqArchiveWriter, Error> {
        let mut entries = self
            .filenames()?
            .iter()
            .map(|name| {
                let blocks = self.get_blocks_for(name)?.ok_or_else(|| {
                    Error::CorruptArchive(format!("missing index entry for {}", name))
                })?;
                Ok((name.clone(), blocks))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        // Sort entries by data offset. This recovers the original
        // order of the files in the block section. Doing this allows us to
        // modify the archive with minimal changes to the final output.
        entries.sort_by_key(|f| {
            self.index
                .get(&FilenameCrc::from(f.0.as_str()))
                .map(|e| e.data_offset)
        });

        // The directory isn't totally necessary but having it allows us to write
        // an archive that hasn't been modified and reproduce the original file at
        // the bit level. This is a nice property for testing.
        let directory =
            self.read_blocks(self.directory.data_offset, self.directory.uncompressed_size)?;

        Ok(EqArchiveWriter::from_existing(
            entries,
            directory,
            self.footer.clone(),
        ))
    }

    fn get_blocks_for(&mut self, filename: &str) -> Result<Option<Vec<Block>>, Error> {
        let crc = FilenameCrc::new(filename);
        let Some(entry) = self.index.get(&crc) else {
            return Ok(None);
        };
        let crc_in_entry = FilenameCrc::from(entry.filename_crc);

        if crc_in_entry != crc {
            return Err(Error::CorruptArchive(format!(
                "CRC mismatch for '{:?}': lookup key {:?} != stored {:?}",
                filename, crc, crc_in_entry
            )));
        }

        self.read_blocks(entry.data_offset, entry.uncompressed_size)
            .map(Some)
    }

    fn read_block_headers(
        &mut self,
        data_offset: u32,
        uncompressed_size: u32,
    ) -> Result<Vec<BlockHeader>, Error> {
        let mut collected: u32 = 0;
        let mut headers = Vec::new();
        self.reader.seek(SeekFrom::Start(data_offset as u64))?;

        while collected < uncompressed_size {
            let header = BlockHeader::read(&mut self.reader)?;
            collected += header.uncompressed_size;
            self.reader
                .seek(SeekFrom::Current(header.compressed_size as i64))?;
            headers.push(header);
        }

        Ok(headers)
    }

    fn read_blocks(
        &mut self,
        data_offset: u32,
        uncompressed_size: u32,
    ) -> Result<Vec<Block>, Error> {
        let max_blocks = (uncompressed_size as usize).div_ceil(BlockHeader::MAX_UNCOMPRESSED_SIZE);
        let mut collected: u32 = 0;
        let mut blocks = Vec::with_capacity(max_blocks);
        let mut block_count = 0;
        self.reader.seek(SeekFrom::Start(data_offset as u64))?;

        while collected < uncompressed_size {
            if block_count > max_blocks {
                return Err(Error::CorruptArchive("too many blocks for entry".into()));
            }

            let block_header = BlockHeader::read(&mut self.reader)?;
            let uncompressed = block_header.uncompressed_size;
            if collected + uncompressed > uncompressed_size {
                return Err(Error::CorruptArchive(
                    "block uncompressed size exceeds expected total".into(),
                ));
            }

            let block = Block::read(block_header, &mut self.reader)?;
            blocks.push(block);
            block_count += 1;
            collected += uncompressed;
        }

        Ok(blocks)
    }
}

fn from_reader<S: Read + Seek>(mut reader: S) -> Result<EqArchiveReader<S>, Error> {
    let header = Header::read(&mut reader)?;
    if header.magic_number != Header::MAGIC_NUMBER {
        return Err(Error::CorruptArchive(
            "header: magic number did not match".into(),
        ));
    }
    if header.version != Header::VERSION {
        return Err(Error::CorruptArchive(format!(
            "header: format version {:#010x} is not supported",
            header.version
        )));
    }

    // Jump past the compressed data blocks to the index section
    reader.seek(SeekFrom::Start(header.index_offset.into()))?;

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let entry_count = u32::from_le_bytes(buf);
    if entry_count > MAX_ENTRY_COUNT {
        return Err(Error::CorruptArchive(format!(
            "exceeds max file entry count of {}",
            MAX_ENTRY_COUNT
        )));
    }

    let entries = (0..entry_count)
        .map(|idx| {
            IndexEntry::read(&mut reader)
                .map_err(|_| Error::CorruptArchive(format!("failed to parse index entry {}", idx)))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let (directory_entries, file_entries): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|e| FilenameCrc::from(e.filename_crc) == FilenameCrc::DIRECTORY);

    let directory = directory_entries
        .into_iter()
        .next()
        .ok_or_else(|| Error::CorruptArchive("failed to find directory entry in index".into()))?;

    let index: HashMap<FilenameCrc, IndexEntry> = file_entries
        .into_iter()
        .map(|entry| (FilenameCrc::from(entry.filename_crc), entry))
        .collect();

    // This swallows all errors while reading the footer
    // and treats it as no footer at all. Not ideal but probably
    // acceptable given how simple the footer is.
    let footer = Footer::read(&mut reader).ok();

    Ok(EqArchiveReader {
        reader,
        index,
        directory,
        footer,
    })
}

pub struct EqFileReader {
    blocks: std::vec::IntoIter<Block>,
    curr: Cursor<Vec<u8>>,
}

impl EqFileReader {
    fn new(blocks: Vec<Block>) -> Result<Self, Error> {
        let mut block_iter = blocks.into_iter();
        let initial_data = match block_iter.next() {
            Some(block) => decompress_block(&block)?,
            None => Cursor::new(Vec::new()),
        };
        Ok(Self {
            blocks: block_iter,
            curr: initial_data,
        })
    }
}

impl Read for EqFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.curr.read(buf)?;
        if n > 0 {
            return Ok(n);
        }

        if let Some(block) = self.blocks.next() {
            self.curr = decompress_block(&block)?;
            self.read(buf)
        } else {
            Ok(0)
        }
    }
}

fn decompress_block(block: &Block) -> std::io::Result<Cursor<Vec<u8>>> {
    let mut data = Vec::with_capacity(block.uncompressed_size as usize);
    ZlibDecoder::new(&block.compressed_data[..]).read_to_end(&mut data)?;
    Ok(Cursor::new(data))
}
