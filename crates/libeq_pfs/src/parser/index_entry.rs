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

    use std::fs::File;
    use std::io::{Cursor, Read};

    // Fixture created with:
    // `dd bs=1 skip=2203555 count=12 if=gfaydark.s3d of=gfaydark/index-entry.bin`

    #[test]
    fn it_reads() {
        let mut fixture = File::open("fixtures/gfaydark/index-entry.bin").unwrap();
        let index_entry = IndexEntry::read(&mut fixture).unwrap();

        assert_eq!(index_entry.filename_crc, 0xffe57ac0);
        assert_eq!(index_entry.data_offset, 0x25fe5);
        assert_eq!(index_entry.uncompressed_size, 0x4438);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/index-entry.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();
        let mut cursor = Cursor::new(&fixture_data);
        let index_entry = IndexEntry::read(&mut cursor).unwrap();

        assert_eq!(index_entry.to_bytes(), fixture_data);
    }
}
