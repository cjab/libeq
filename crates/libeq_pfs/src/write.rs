use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::PfsReader;
use crate::crc::FilenameCrc;
use crate::error::Error;
use crate::parser::{BlockHeader, Directory, Footer, Header, IndexEntry};

pub struct PfsWriter<W> {
    pub(crate) writer: W,
    pub(crate) entries: Vec<(String, IndexEntry)>,
    pub(crate) directory: Option<IndexEntry>,
    pub(crate) footer: Option<Footer>,
}

//----------------------
// Public API
//----------------------
impl<W: Read + Write + Seek> PfsWriter<W> {
    pub fn create(writer: W) -> Result<Self, Error> {
        let mut new = Self {
            writer,
            entries: Vec::new(),
            directory: None,
            footer: None,
        };
        // Write the default header on creation
        new.write_header()?;
        Ok(new)
    }

    pub fn from_reader<R: Read + Seek>(
        reader: &mut PfsReader<R>,
        writer: W,
    ) -> Result<Self, Error> {
        reader.to_writer(writer)
    }

    pub fn insert(
        &mut self,
        filename: impl Into<String>,
        mut reader: impl Read,
    ) -> Result<bool, Error> {
        let filename = filename.into();
        // Does this filename already exist in the archive?
        let replaced = self.entries.iter().any(|(f, _)| f == &filename);
        if replaced {
            self.entries.retain(|(f, _)| f != &filename)
        };
        // Clear the directory, inserting a file invalidated it.
        // We now need to generate a new one at finish.
        self.directory = None;

        let data_offset = self.writer.stream_position()? as u32;
        let uncompressed_size = self.write_file(&mut reader)?;
        let filename_crc = FilenameCrc::new(&filename).into();
        self.entries.push((
            filename,
            IndexEntry {
                data_offset,
                uncompressed_size,
                filename_crc,
            },
        ));
        Ok(replaced)
    }

    pub fn remove(&mut self, filename: &str) -> bool {
        let removed = self.entries.iter().any(|(f, _)| f == filename);
        if removed {
            self.entries.retain(|(f, _)| f != filename);
            // Clear the directory, removing a file invalidated it.
            // We now need to generate a new one at finish.
            self.directory = None;
        };
        removed
    }

    pub fn filenames(&self) -> Vec<String> {
        self.entries.iter().map(|(name, _)| name.clone()).collect()
    }

    pub fn finish(mut self) -> Result<W, Error> {
        let was_modified = self.directory.is_none();
        // At this point we assume that the header and all file blocks
        // have been written. We just need to write the directory, index,
        // and footer.
        self.write_directory()?;
        self.write_index()?;
        self.write_footer(was_modified)?;
        Ok(self.writer)
    }
}

//----------------------
// Write operations
//----------------------
impl<W: Read + Write + Seek> PfsWriter<W> {
    fn write_header(&mut self) -> Result<(), Error> {
        self.writer.write_all(&Header::default().to_bytes())?;
        Ok(())
    }

    fn write_file(&mut self, reader: &mut impl Read) -> Result<u32, Error> {
        let mut chunk = Vec::with_capacity(BlockHeader::MAX_UNCOMPRESSED_SIZE);
        let mut buf = Vec::with_capacity(BlockHeader::MAX_COMPRESSED_SIZE);
        let mut file_size = 0;
        loop {
            chunk.clear();
            let uncompressed_size = reader
                .take(BlockHeader::MAX_UNCOMPRESSED_SIZE as u64)
                .read_to_end(&mut chunk)? as u32;
            file_size += uncompressed_size;
            if uncompressed_size == 0 {
                return Ok(file_size);
            }
            buf.clear();
            let mut encoder = ZlibEncoder::new(&mut buf, Compression::default());
            encoder.write_all(&chunk)?;
            let compressed = encoder.finish()?;
            let header = BlockHeader {
                uncompressed_size,
                compressed_size: compressed.len() as u32,
            };
            self.writer.write_all(&header.to_bytes())?;
            self.writer.write_all(compressed)?;
        }
    }

    fn write_directory(&mut self) -> Result<(), Error> {
        if self.directory.is_some() {
            // We have an existing directory that hasn't been invalidated.
            // That means it should already be at the end of block section,
            // we don't need to do anything!
            return Ok(());
        };

        // Otherwise we need to create and write the directory file
        let mut filenames = self.filenames();
        filenames.sort_by_key(|f| FilenameCrc::new(f));
        let dir = Directory { filenames }.to_bytes();
        let data_offset = self.writer.stream_position()? as u32;
        let mut cursor = Cursor::new(dir);
        let uncompressed_size = self.write_file(&mut cursor)?;

        // And also create the associated index,
        // to be written later in the index section by write_index.
        self.directory = Some(IndexEntry {
            uncompressed_size,
            data_offset,
            filename_crc: FilenameCrc::DIRECTORY.into(),
        });

        Ok(())
    }

    fn write_index(&mut self) -> Result<(), Error> {
        // Update the index pointer in the file header
        let index_offset = self.writer.stream_position()? as u32;
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(&index_offset.to_le_bytes())?;
        self.writer.seek(SeekFrom::Start(index_offset as u64))?;

        let Some(dir) = self.directory else {
            // This means that for whatever reason we tried to write
            // the index before writing the directory file.
            return Err(Error::CorruptArchive(
                "directory index does not exist".into(),
            ));
        };

        let mut all_entries: Vec<_> = self.entries.iter().map(|(_, e)| *e).collect();
        all_entries.push(dir);
        all_entries.sort_by_key(|e| e.filename_crc);

        // Then seek forward again and write the entry/file count,
        let entry_count = all_entries.len() as u32;
        self.writer.write_all(&entry_count.to_le_bytes())?;

        // Followed by all index entries
        for entry in all_entries.iter() {
            self.writer.write_all(&entry.to_bytes())?;
        }

        Ok(())
    }

    fn write_footer(&mut self, was_modified: bool) -> Result<(), Error> {
        let bytes = match (was_modified, &self.footer) {
            // Preserve the footer if it exists and no changes have been made
            (false, Some(f)) => f.to_bytes(),
            // Update the timestamp if footer exists and changes have been made
            (true, Some(f)) => Footer {
                footer_string: f.footer_string,
                timestamp: current_unix_timestamp(),
            }
            .to_bytes(),
            // If the original archive had no footer do not add one
            (false, None) => vec![],
            // This is a new archive may as well add a footer
            (true, None) => Footer {
                footer_string: Footer::FOOTER_STRING,
                timestamp: current_unix_timestamp(),
            }
            .to_bytes(),
        };
        self.writer.write_all(&bytes)?;
        Ok(())
    }
}

fn current_unix_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}
