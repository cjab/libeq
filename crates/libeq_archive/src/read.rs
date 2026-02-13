use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use flate2::read::ZlibDecoder;

use crate::crc::FilenameCrc;
use crate::error::Error;
use crate::parser::{Directory, Header, IndexEntry};

const UNCOMPRESSED_BLOCK_SIZE: usize = 8192;

pub struct EqArchiveReader<R> {
    reader: R,
    index: HashMap<FilenameCrc, IndexEntry>,
    directory: IndexEntry,
}

impl EqArchiveReader<Cursor<Vec<u8>>> {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut cursor = Cursor::new(buf);
        let (index, directory) = parse_index(&mut cursor)?;

        Ok(EqArchiveReader {
            reader: cursor,
            index,
            directory,
        })
    }
}

impl<R: Read + Seek> EqArchiveReader<R> {
    pub fn open(mut reader: R) -> Result<Self, Error> {
        let (index, directory) = parse_index(&mut reader)?;
        Ok(EqArchiveReader {
            reader,
            index,
            directory,
        })
    }

    pub fn get(&mut self, filename: &str) -> Result<Option<Vec<u8>>, Error> {
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

        self.decompress_entry(entry.data_offset, entry.uncompressed_size)
    }

    pub fn filenames(&mut self) -> Result<Vec<String>, Error> {
        let IndexEntry {
            data_offset,
            uncompressed_size,
            ..
        } = self.directory;
        let data = self
            .decompress_entry(data_offset, uncompressed_size)?
            .ok_or_else(|| Error::CorruptArchive("directory data not found".into()))?;
        let (_, directory) = Directory::parse(&data[..])
            .map_err(|_| Error::CorruptArchive("failed to parse directory".into()))?;
        Ok(directory.filenames)
    }

    fn decompress_entry(
        &mut self,
        data_offset: u32,
        uncompressed_size: u32,
    ) -> Result<Option<Vec<u8>>, Error> {
        let max_blocks =
            (uncompressed_size as usize + UNCOMPRESSED_BLOCK_SIZE - 1) / UNCOMPRESSED_BLOCK_SIZE;
        let mut block_count = 0;
        let mut decompressed = Vec::with_capacity(uncompressed_size as usize);
        self.reader.seek(SeekFrom::Start(data_offset as u64))?;

        while (decompressed.len() as u32) < uncompressed_size {
            if block_count > max_blocks {
                return Err(Error::CorruptArchive("too many blocks for entry".into()));
            }

            let mut buf = [0u8; 4];
            self.reader.read_exact(&mut buf)?;
            let block_compressed_size = u32::from_le_bytes(buf);
            self.reader.read_exact(&mut buf)?;
            let block_uncompressed_size = u32::from_le_bytes(buf);

            if decompressed.len() as u32 + block_uncompressed_size > uncompressed_size {
                return Err(Error::CorruptArchive(
                    "block uncompressed size exceeds expected total".into(),
                ));
            }

            let mut compressed_block = vec![0u8; block_compressed_size as usize];
            self.reader.read_exact(&mut compressed_block)?;
            let mut block = Vec::with_capacity(block_uncompressed_size as usize);
            ZlibDecoder::new(&compressed_block[..]).read_to_end(&mut block)?;
            block_count += 1;
            decompressed.extend_from_slice(&block);
        }

        Ok(Some(decompressed))
    }
}

fn parse_index<S: Read + Seek>(
    reader: &mut S,
) -> Result<(HashMap<FilenameCrc, IndexEntry>, IndexEntry), Error> {
    let mut buf = [0u8; Header::SIZE as usize];
    reader.read_exact(&mut buf)?;
    let (_, header) =
        Header::parse(&buf).map_err(|_| Error::CorruptArchive("failed to parse header".into()))?;

    if header.magic_number != Header::MAGIC_NUMBER {
        return Err(Error::CorruptArchive("magic number did not match".into()));
    }

    if header.version != Header::VERSION {
        return Err(Error::CorruptArchive(format!(
            "format version {:#010x} is not supported",
            header.version
        )));
    }

    reader.seek(SeekFrom::Start(header.index_offset.into()))?;
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let entry_count = u32::from_le_bytes(buf);

    let mut buf = vec![0u8; entry_count as usize * IndexEntry::SIZE as usize];
    reader.read_exact(&mut buf)?;

    let entries: Vec<IndexEntry> = buf
        .chunks(12)
        .enumerate()
        .map(|(idx, chunk)| {
            IndexEntry::parse(chunk)
                .map(|(_, e)| e)
                .map_err(|_| Error::CorruptArchive(format!("failed to parse index entry {}", idx)))
        })
        .collect::<Result<_, _>>()?;

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

    Ok((index, directory))
}
