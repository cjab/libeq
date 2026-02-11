use nom::IResult;
use nom::number::complete::le_u32;

#[derive(Debug, Default)]
pub struct IndexEntry {
    pub filename_crc: u32,
    pub data_offset: u32,
    pub uncompressed_size: u32,
}

impl IndexEntry {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, filename_crc) = le_u32(input)?;
        let (i, data_offset) = le_u32(i)?;
        let (i, uncompressed_size) = le_u32(i)?;

        Ok((
            i,
            IndexEntry {
                uncompressed_size,
                filename_crc,
                data_offset,
            },
        ))
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
    use std::io::Read;

    // Fixture created with:
    // `dd bs=1 skip=2203555 count=12 if=gfaydark.s3d of=gfaydark/index-entry.bin`

    #[test]
    fn it_parses() {
        let mut fixture = File::open("fixtures/gfaydark/index-entry.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, index_entry) = IndexEntry::parse(&fixture_data).unwrap();

        assert_eq!(index_entry.filename_crc, 0xffe57ac0);
        assert_eq!(index_entry.data_offset, 0x25fe5);
        assert_eq!(index_entry.uncompressed_size, 0x4438);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/index-entry.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, index_entry) = IndexEntry::parse(&fixture_data).unwrap();

        assert_eq!(index_entry.to_bytes(), fixture_data);
    }
}
