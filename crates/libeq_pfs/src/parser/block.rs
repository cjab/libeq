use std::io::Read;

use crate::error::Error;
use crate::parser::BlockHeader;

#[derive(Debug, Default)]
pub struct Block {
    pub uncompressed_size: u32,
    pub compressed_data: Vec<u8>,
}

impl Block {
    pub fn read(header: BlockHeader, reader: &mut impl Read) -> Result<Self, Error> {
        let mut compressed_data = vec![0u8; header.compressed_size as usize];
        reader.read_exact(&mut compressed_data)?;

        Ok(Self {
            uncompressed_size: header.uncompressed_size,
            compressed_data,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &(self.compressed_data.len() as u32).to_le_bytes()[..],
            &self.uncompressed_size.to_le_bytes(),
            &self.compressed_data,
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
    // `dd bs=1 skip=155621 count=7201 if=gfaydark.s3d of=gfaydark/block.bin`
    // Header + Compressed Data = 8 + 7193 = 7201 bytes

    #[test]
    fn it_reads() {
        let mut fixture = File::open("fixtures/gfaydark/block.bin").unwrap();
        let block_header = BlockHeader::read(&mut fixture).unwrap();
        let block = Block::read(block_header, &mut fixture).unwrap();

        assert_eq!(block.uncompressed_size, 0x2000);
        assert_eq!(block.compressed_data.len(), 0x1c19);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/block.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();
        let mut cursor = Cursor::new(&fixture_data);
        let block_header = BlockHeader::read(&mut cursor).unwrap();
        let block = Block::read(block_header, &mut cursor).unwrap();

        assert_eq!(block.to_bytes(), fixture_data);
    }
}
