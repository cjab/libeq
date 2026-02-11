use nom::IResult;
use nom::Parser;
use nom::bytes::complete::take;
use nom::number::complete::le_u32;

#[derive(Debug, Default)]
pub struct Block {
    pub uncompressed_size: u32,
    pub compressed_data: Vec<u8>,
}

impl Block {
    pub const HEADER_SIZE: usize = 8;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, compressed_size) = le_u32(input)?;
        let (i, uncompressed_size) = le_u32(i)?;
        let (i, compressed_data) = take(compressed_size).parse(i)?;

        Ok((
            i,
            Self {
                uncompressed_size,
                compressed_data: Vec::from(compressed_data),
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &(self.compressed_data.len() as u32).to_le_bytes()[..],
            &self.uncompressed_size.to_le_bytes(),
            &self.compressed_data,
        ]
        .concat()
    }

    pub fn size(&self) -> usize {
        self.compressed_data.len() + Self::HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Read;

    // Fixture created with:
    // `dd bs=1 skip=155621 count=7201 if=gfaydark.s3d of=gfaydark/block.bin`
    // Header + Compressed Data = 8 + 7193 = 7201 bytes

    #[test]
    fn it_parses() {
        let mut fixture = File::open("fixtures/gfaydark/block.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, block) = Block::parse(&fixture_data).unwrap();

        assert_eq!(block.uncompressed_size, 0x2000);
        assert_eq!(block.compressed_data.len(), 0x1c19);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/block.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, block) = Block::parse(&fixture_data).unwrap();

        assert_eq!(block.to_bytes(), fixture_data);
    }
}
