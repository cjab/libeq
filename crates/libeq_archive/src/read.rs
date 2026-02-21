use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use flate2::read::ZlibDecoder;

use crate::crc::FilenameCrc;
use crate::error::Error;
use crate::parser::{Block, BlockHeader, Directory, Footer, Header, IndexEntry};
use crate::write::EqArchiveWriter;

const MAX_ENTRY_COUNT: u32 = 100_000; // 100k files

#[derive(Debug, Eq, PartialEq)]
pub struct FileInfo {
    pub data_offset: u32,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub block_count: u32,
}

#[derive(Debug)]
pub struct ArchiveInfo {
    pub version: u32,
    pub index_offset: u32,
    pub file_count: u32,
    pub footer_string: Option<[u8; 5]>,
    pub timestamp: Option<u32>,
}

pub struct EqArchiveReader<R> {
    reader: R,
    index: HashMap<FilenameCrc, IndexEntry>,
    directory: IndexEntry,
    footer: Option<Footer>,
}

//----------------------
// Public API
//----------------------
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

    pub fn archive_info(&mut self) -> Result<ArchiveInfo, Error> {
        self.reader.seek(SeekFrom::Start(0))?;
        let header = Header::read(&mut self.reader)?;
        Ok(ArchiveInfo {
            version: header.version,
            index_offset: header.index_offset,
            file_count: self.index.len() as u32,
            footer_string: self.footer.as_ref().map(|f| f.footer_string),
            timestamp: self.footer.as_ref().map(|f| f.timestamp),
        })
    }

    pub fn get(&mut self, filename: &str) -> Result<Option<Vec<u8>>, Error> {
        let Some(mut reader) = self.get_reader(filename)? else {
            return Ok(None);
        };
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(Some(buf))
    }

    pub fn get_reader(&mut self, filename: &str) -> Result<Option<impl Read>, Error> {
        let Some(entry) = self.get_index_entry(filename)? else {
            return Ok(None);
        };
        Ok(Some(EqFileReader::new(self.iter_blocks(&entry)?)?))
    }

    pub fn info(&mut self, filename: &str) -> Result<Option<FileInfo>, Error> {
        let Some(entry) = self.get_index_entry(filename)?.map(|x| x.clone()) else {
            return Ok(None);
        };
        let headers: Vec<_> = self
            .iter_block_headers(&entry)?
            .collect::<Result<_, Error>>()?;

        Ok(Some(FileInfo {
            data_offset: entry.data_offset,
            compressed_size: headers.iter().map(|h| h.compressed_size).sum(),
            uncompressed_size: entry.uncompressed_size,
            block_count: headers.len() as u32,
        }))
    }

    pub fn directory_info(&mut self) -> Result<FileInfo, Error> {
        let dir = self.directory;
        let headers: Vec<_> = self
            .iter_block_headers(&dir)?
            .collect::<Result<_, Error>>()?;
        Ok(FileInfo {
            data_offset: dir.data_offset,
            compressed_size: headers.iter().map(|h| h.compressed_size).sum(),
            uncompressed_size: dir.uncompressed_size,
            block_count: headers.len() as u32,
        })
    }

    pub fn filenames(&mut self) -> Result<Vec<String>, Error> {
        let dir = self.directory;
        let dir_size = dir.uncompressed_size;
        let blocks = self.iter_blocks(&dir)?;
        let mut data = Vec::with_capacity(dir_size as usize);
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
                let Some(entry) = self.get_index_entry(name)? else {
                    return Err(Error::CorruptArchive(format!(
                        "missing index entry for {}",
                        name
                    )));
                };
                let blocks = self.read_blocks(entry)?;
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
        let directory = self.read_blocks(self.directory)?;

        Ok(EqArchiveWriter::from_existing(
            entries,
            directory,
            self.footer.clone(),
        ))
    }
}

//----------------------
// Block iters
//----------------------
struct BlockIterState<'a, R> {
    reader: &'a mut R,
    blocks_read: usize,
    max_blocks: usize,
    uncompressed_read: usize,
    uncompressed: usize,
}

impl<'a, R: Read + Seek> BlockIterState<'a, R> {
    fn new(entry: &IndexEntry, reader: &'a mut R) -> Self {
        BlockIterState {
            reader,
            blocks_read: 0,
            max_blocks: (entry.uncompressed_size as usize)
                .div_ceil(BlockHeader::MAX_UNCOMPRESSED_SIZE),
            uncompressed_read: 0,
            uncompressed: entry.uncompressed_size as usize,
        }
    }

    fn next_header(&mut self) -> Option<Result<BlockHeader, Error>> {
        let remaining = self.uncompressed.saturating_sub(self.uncompressed_read);
        if remaining == 0 {
            return None;
        }

        let header = match BlockHeader::read(&mut self.reader) {
            Ok(h) => h,
            Err(e) => return Some(Err(e)),
        };
        self.blocks_read += 1;
        self.uncompressed_read = self
            .uncompressed_read
            .saturating_add(header.uncompressed_size as usize);

        if (header.uncompressed_size as usize) > remaining {
            return Some(Err(Error::CorruptArchive(
                "block uncompressed size exceeds expected total".into(),
            )));
        }
        if self.blocks_read > self.max_blocks {
            return Some(Err(Error::CorruptArchive(
                "too many blocks for entry".into(),
            )));
        }

        Some(Ok(header))
    }
}

pub struct BlockHeaderIter<'a, R>(BlockIterState<'a, R>);

impl<'a, R: Read + Seek> Iterator for BlockHeaderIter<'a, R> {
    type Item = Result<BlockHeader, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let header = match self.0.next_header()? {
            Ok(h) => h,
            Err(e) => return Some(Err(e)),
        };
        // Seek past the compressed data
        if let Err(e) = self
            .0
            .reader
            .seek(SeekFrom::Current(header.compressed_size as i64))
        {
            return Some(Err(e.into()));
        };
        Some(Ok(header))
    }
}

pub struct BlockIter<'a, R>(BlockIterState<'a, R>);

impl<'a, R: Read + Seek> Iterator for BlockIter<'a, R> {
    type Item = Result<Block, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let header = match self.0.next_header()? {
            Ok(h) => h,
            Err(e) => return Some(Err(e)),
        };
        let block = match Block::read(header, &mut self.0.reader) {
            Ok(h) => h,
            Err(e) => return Some(Err(e)),
        };
        Some(Ok(block))
    }
}

//----------------------
// Block operations
//----------------------
impl<R: Read + Seek> EqArchiveReader<R> {
    fn get_index_entry(&self, filename: &str) -> Result<Option<IndexEntry>, Error> {
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

        Ok(Some(*entry))
    }

    fn iter_block_headers(&mut self, entry: &IndexEntry) -> Result<BlockHeaderIter<'_, R>, Error> {
        self.reader
            .seek(SeekFrom::Start(entry.data_offset as u64))?;
        Ok(BlockHeaderIter(BlockIterState::new(
            entry,
            &mut self.reader,
        )))
    }

    fn iter_blocks(&mut self, entry: &IndexEntry) -> Result<BlockIter<'_, R>, Error> {
        self.reader
            .seek(SeekFrom::Start(entry.data_offset as u64))?;
        Ok(BlockIter(BlockIterState::new(entry, &mut self.reader)))
    }

    fn read_blocks(&mut self, entry: IndexEntry) -> Result<Vec<Block>, Error> {
        self.iter_blocks(&entry)?.collect()
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

pub struct EqFileReader<I: Iterator<Item = Result<Block, Error>>> {
    blocks: I,
    curr: Cursor<Vec<u8>>,
}

impl<I: Iterator<Item = Result<Block, Error>>> EqFileReader<I> {
    fn new(mut blocks: I) -> Result<Self, Error> {
        let initial_data = match blocks.next() {
            Some(block) => decompress_block(&block?)?,
            None => Cursor::new(Vec::new()),
        };
        Ok(Self {
            blocks,
            curr: initial_data,
        })
    }
}

impl<I: Iterator<Item = Result<Block, Error>>> Read for EqFileReader<I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.curr.read(buf)?;
        if n > 0 {
            return Ok(n);
        }

        if let Some(block) = self.blocks.next() {
            self.curr = decompress_block(&block?)?;
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
