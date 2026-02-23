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
    use std::io::Cursor;

    fn fixture() -> BlockHeader {
        BlockHeader {
            compressed_size: 100,
            uncompressed_size: 200,
        }
    }

    #[test]
    fn it_reads() {
        let data = fixture().to_bytes();
        let header = BlockHeader::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(header.compressed_size, 100);
        assert_eq!(header.uncompressed_size, 200);
    }

    #[test]
    fn it_serializes() {
        let data = fixture().to_bytes();
        let header = BlockHeader::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(header.to_bytes(), data);
    }
}
