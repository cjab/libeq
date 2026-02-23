use std::io::Read;

use crate::error::Error;

#[derive(Debug, Default, Copy, Clone)]
pub struct IndexEntry {
    pub filename_crc: u32,
    pub data_offset: u32,
    pub uncompressed_size: u32,
}

impl IndexEntry {
    // All original .s3d files that I have seen use this as the CRC
    // for the directory file. I have no idea what string it corresponds to
    // (if any). The EQZip tool appears to use 0xffffffff in place of this.
    // So any archives written with that tool will currently fail.
    pub const DIRECTORY_CRC: u32 = 0x61580ac9;

    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let filename_crc = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let data_offset = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let uncompressed_size = u32::from_le_bytes(buf);

        Ok(Self {
            filename_crc,
            data_offset,
            uncompressed_size,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &self.filename_crc.to_le_bytes()[..],
            &self.data_offset.to_le_bytes(),
            &self.uncompressed_size.to_le_bytes(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> IndexEntry {
        IndexEntry {
            filename_crc: 0xAABBCCDD,
            data_offset: 0x1000,
            uncompressed_size: 0x2000,
        }
    }

    #[test]
    fn it_reads() {
        let data = fixture().to_bytes();
        let entry = IndexEntry::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(entry.filename_crc, 0xAABBCCDD);
        assert_eq!(entry.data_offset, 0x1000);
        assert_eq!(entry.uncompressed_size, 0x2000);
    }

    #[test]
    fn it_serializes() {
        let data = fixture().to_bytes();
        let entry = IndexEntry::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(entry.to_bytes(), data);
    }
}
