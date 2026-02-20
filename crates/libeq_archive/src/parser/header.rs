use std::io::Read;

use crate::error::Error;

#[derive(Debug, PartialEq)]
pub struct Header {
    pub index_offset: u32,
    pub magic_number: u32,
    pub version: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            index_offset: 0,
            magic_number: Self::MAGIC_NUMBER,
            version: Self::VERSION,
        }
    }
}

impl Header {
    pub const MAGIC_NUMBER: u32 = u32::from_le_bytes(*b"PFS ");
    pub const VERSION: u32 = 0x00020000;
    pub const SIZE: u32 = 12;

    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let index_offset = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let magic_number = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let version = u32::from_le_bytes(buf);

        Ok(Self {
            index_offset,
            magic_number,
            version,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &self.index_offset.to_le_bytes()[..],
            &self.magic_number.to_le_bytes()[..],
            &self.version.to_le_bytes()[..],
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
    // `dd bs=1 count=12 if=gfaydark.s3d of=gfaydark/header.bin`

    #[test]
    fn it_reads() {
        let mut fixture = File::open("fixtures/gfaydark/header.bin").unwrap();
        let header = Header::read(&mut fixture).unwrap();

        assert_eq!(header.index_offset, 0x219dbf);
        assert_eq!(header.magic_number, u32::from_le_bytes(*b"PFS "));
        assert_eq!(header.version, 0x00020000);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/header.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();
        let mut cursor = Cursor::new(&fixture_data);
        let header = Header::read(&mut cursor).unwrap();

        assert_eq!(header.to_bytes(), fixture_data);
    }
}
