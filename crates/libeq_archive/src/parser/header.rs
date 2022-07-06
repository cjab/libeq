use nom::number::complete::le_u32;
use nom::IResult;

#[derive(Debug, Default, PartialEq)]
pub struct Header {
    pub index_offset: u32,
    pub magic_number: u32,
    pub version: u32,
}

impl Header {
    pub const MAGIC_NUMBER: u32 = u32::from_le_bytes(*b"PFS ");
    pub const VERSION: u32 = 0x00020000;
    pub const SIZE: usize = 12;

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, index_offset) = le_u32(input)?;
        let (i, magic_number) = le_u32(i)?;
        let (i, version) = le_u32(i)?;

        Ok((
            i,
            Header {
                index_offset,
                magic_number,
                version,
            },
        ))
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
    use std::io::Read;

    // Fixture created with:
    // `dd bs=1 count=12 if=gfaydark.s3d of=gfaydark/header.bin`

    #[test]
    fn it_parses() {
        let mut fixture = File::open("fixtures/gfaydark/header.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, header) = Header::parse(&fixture_data).unwrap();

        assert_eq!(header.index_offset, 0x219dbf);
        assert_eq!(header.magic_number, u32::from_le_bytes(*b"PFS "));
        assert_eq!(header.version, 0x00020000);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/header.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, header) = Header::parse(&fixture_data).unwrap();

        assert_eq!(header.to_bytes(), fixture_data);
    }
}
