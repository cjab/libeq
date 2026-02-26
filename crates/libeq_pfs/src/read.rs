use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use flate2::read::ZlibDecoder;

use crate::crc::FilenameCrc;
use crate::error::Error;
use crate::parser::{Block, BlockHeader, Directory, Footer, Header, IndexEntry};
use crate::write::PfsWriter;

const MAX_ENTRY_COUNT: u32 = 100_000; // 100k files

/// Information about a file stored in the archive.
#[derive(Debug, Eq, PartialEq)]
pub struct FileInfo {
    /// The offset to the file's data in the data block section.
    pub data_offset: u32,
    /// The total size of the file as compressed in the archive.
    pub compressed_size: u32,
    /// The total size of the file after decompression.
    pub uncompressed_size: u32,
    /// The total number of compressed blocks the file is stored in.
    pub block_count: u32,
}

/// Information about the overall PFS archive itself.
#[derive(Debug)]
pub struct PfsInfo {
    /// The PFS version word stored in the header.
    pub version: u32,
    /// The offset to the beginning of the index section of the archive.
    pub index_offset: u32,
    /// The number of files in this archive.
    pub file_count: u32,
    /// The string stored in the (optional) footer (nearly always STEVE).
    pub footer_string: Option<[u8; 5]>,
    /// The timestamp stored in the (optional) footer.
    pub timestamp: Option<u32>,
}

/// Reader of a PFS file archive.
pub struct PfsReader<R> {
    reader: R,
    index: HashMap<FilenameCrc, IndexEntry>,
    directory: IndexEntry,
    footer: Option<Footer>,
}

/// Buffered reading of PFS files
impl PfsReader<Cursor<Vec<u8>>> {
    /// Read the entire PFS file into memory and return a PfsReader.
    ///
    /// This is useful for small PFS files but you will want to use
    /// [`PfsReader::open`] if memory efficiency is important.
    pub fn read<R: Read>(mut reader: R) -> Result<Self, Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        from_reader(Cursor::new(buf))
    }
}

/// Public API for reading PFS files
impl<R: Read + Seek> PfsReader<R> {
    /// Open the PFS file and return a PfsReader.
    ///
    /// This will read the minimal amount of PFS file data into memory
    /// and lazily read files from disk as needed. (e.g. when [`PfsReader::get`]
    /// is called).
    pub fn open(reader: R) -> Result<Self, Error> {
        from_reader(reader)
    }

    /// Read information about the overall PFS file.
    pub fn archive_info(&mut self) -> Result<PfsInfo, Error> {
        self.reader.seek(SeekFrom::Start(0))?;
        let header = Header::read(&mut self.reader)?;
        Ok(PfsInfo {
            version: header.version,
            index_offset: header.index_offset,
            file_count: self.index.len() as u32,
            footer_string: self.footer.as_ref().map(|f| f.footer_string),
            timestamp: self.footer.as_ref().map(|f| f.timestamp),
        })
    }

    /// Get file data from the archive by filename.
    pub fn get(&mut self, filename: &str) -> Result<Option<Vec<u8>>, Error> {
        let Some(mut reader) = self.get_reader(filename)? else {
            return Ok(None);
        };
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(Some(buf))
    }

    /// Get a reader for a file in the archive by filename.
    ///
    /// This avoids reading the entire file into memory at once,
    /// unlike [`PfsReader::get`].
    pub fn get_reader(&mut self, filename: &str) -> Result<Option<impl Read>, Error> {
        let Some(entry) = self.get_index_entry(filename)? else {
            return Ok(None);
        };
        Ok(Some(PfsFileReader::new(self.iter_blocks(&entry)?)?))
    }

    /// Get information about a file in the archive by filename.
    pub fn info(&mut self, filename: &str) -> Result<Option<FileInfo>, Error> {
        let Some(entry) = self.get_index_entry(filename)? else {
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

    /// Get information about the archive file directory.
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

    /// List all files in the archive.
    pub fn filenames(&mut self) -> Result<Vec<String>, Error> {
        let dir = self.directory;
        let dir_size = dir.uncompressed_size;
        let blocks = self.iter_blocks(&dir)?;
        let mut data = Vec::with_capacity(dir_size as usize);
        let mut reader = PfsFileReader::new(blocks)?;
        reader.read_to_end(&mut data)?;
        let mut cursor = Cursor::new(data);
        let directory = Directory::read(&mut cursor)?;
        Ok(directory.filenames)
    }

    /// Create a new PfsWriter from this PfsReader.
    ///
    /// This copies all files from the reader into the provided `dest`
    /// writer. The writer then becomes the backing writer for the newly
    /// created [`PfsWriter`].
    pub fn to_writer<W: Read + Write + Seek>(&mut self, dest: W) -> Result<PfsWriter<W>, Error> {
        let mut entries = self
            .filenames()?
            .into_iter()
            .map(|name| {
                let entry = self.get_index_entry(&name)?.ok_or_else(|| {
                    Error::CorruptArchive(format!("missing index entry for {}", name))
                })?;
                Ok((name, entry))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        // Sort entries by data offset. This recovers the original
        // order of the files in the block section. Doing this allows us to
        // modify the archive with minimal changes to the final output.
        entries.sort_by_key(|(_, e)| e.data_offset);

        let mut writer = PfsWriter::create(dest)?;
        writer.entries = entries;
        writer.footer = self.footer;
        for (_, e) in &writer.entries {
            for b in self.iter_blocks(e)? {
                writer.writer.write_all(&b?.to_bytes())?;
            }
        }
        let directory = self.directory;
        for b in self.iter_blocks(&directory)? {
            writer.writer.write_all(&b?.to_bytes())?;
        }
        writer.directory = Some(directory);
        Ok(writer)
    }
}

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

struct BlockHeaderIter<'a, R>(BlockIterState<'a, R>);

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

struct BlockIter<'a, R>(BlockIterState<'a, R>);

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

/// Internal block-level reader operations
impl<R: Read + Seek> PfsReader<R> {
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
}

fn from_reader<S: Read + Seek>(mut reader: S) -> Result<PfsReader<S>, Error> {
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

    Ok(PfsReader {
        reader,
        index,
        directory,
        footer,
    })
}

/// Lazily read a file stored in a PFS archive.
pub struct PfsFileReader<I: Iterator<Item = Result<Block, Error>>> {
    blocks: I,
    curr: Cursor<Vec<u8>>,
}

impl<I: Iterator<Item = Result<Block, Error>>> PfsFileReader<I> {
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

impl<I: Iterator<Item = Result<Block, Error>>> Read for PfsFileReader<I> {
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
