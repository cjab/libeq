use std::io::Read;

use crate::error::Error;

#[derive(Debug, Default)]
pub struct BlockHeader {
    pub compressed_size: u32,
    pub uncompressed_size: u32,
}

impl BlockHeader {
    pub const MAX_UNCOMPRESSED_SIZE: usize = 8 * 1024; // 8KB
    // The compressed size should rarely be larger than the uncompressed,
    // but it could happen. Add a small buffer here for that case.
    pub const MAX_COMPRESSED_SIZE: usize = Self::MAX_UNCOMPRESSED_SIZE + 128;

    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let compressed_size = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let uncompressed_size = u32::from_le_bytes(buf);

        if compressed_size as usize > Self::MAX_COMPRESSED_SIZE {
            return Err(Error::CorruptArchive(
                "block compressed size too large".into(),
            ));
        }
        if uncompressed_size as usize > Self::MAX_UNCOMPRESSED_SIZE {
            return Err(Error::CorruptArchive(
                "block uncompressed size too large".into(),
            ));
        }

        Ok(Self {
            compressed_size,
            uncompressed_size,
        })
    }

    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &self.compressed_size.to_le_bytes()[..],
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
    // `dd bs=1 skip=155621 count=8 if=gfaydark.s3d of=gfaydark/block.bin`

    #[test]
    fn it_reads() {
        let mut fixture = File::open("fixtures/gfaydark/block-header.bin").unwrap();
        let block_header = BlockHeader::read(&mut fixture).unwrap();

        assert_eq!(block_header.uncompressed_size, 0x2000);
        assert_eq!(block_header.compressed_size, 0x1c19);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/block-header.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();
        let mut cursor = Cursor::new(&fixture_data);
        let block_header = BlockHeader::read(&mut cursor).unwrap();

        assert_eq!(block_header.to_bytes(), fixture_data);
    }
}
