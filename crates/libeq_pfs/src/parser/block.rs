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
    use std::io::Cursor;

    fn fixture() -> Block {
        Block {
            uncompressed_size: 10,
            compressed_data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        }
    }

    #[test]
    fn it_reads() {
        let bytes = fixture().to_bytes();
        // Block::read takes a pre-parsed header and reads compressed_data from the reader
        let header = BlockHeader {
            compressed_size: 4,
            uncompressed_size: 10,
        };
        // Skip past the header bytes (first 8 bytes) in the serialized data
        let block = Block::read(header, &mut Cursor::new(&bytes[8..])).unwrap();

        assert_eq!(block.uncompressed_size, 10);
        assert_eq!(block.compressed_data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn it_serializes() {
        let data = fixture().to_bytes();
        let header = BlockHeader {
            compressed_size: 4,
            uncompressed_size: 10,
        };
        let block = Block::read(header, &mut Cursor::new(&data[8..])).unwrap();

        assert_eq!(block.to_bytes(), data);
    }
}
