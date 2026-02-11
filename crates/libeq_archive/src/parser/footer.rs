use nom::IResult;
use nom::Parser;
use nom::bytes::complete::take;
use nom::number::complete::le_u32;

#[derive(Debug, Default, PartialEq)]
pub struct Footer {
    pub footer_string: Vec<u8>,
    // Is this _really_ a timestamp?
    pub timestamp: u32,
}

impl Footer {
    pub const FOOTER_STRING: [u8; 5] = *b"STEVE";

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, footer_string) = take(5usize).parse(input)?;
        let (i, timestamp) = le_u32(i)?;

        Ok((
            i,
            Footer {
                footer_string: Vec::from(footer_string),
                timestamp,
            },
        ))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [&self.footer_string[..], &self.timestamp.to_le_bytes()].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Read;

    // Fixture created with:
    // `dd bs=1 skip=2203567 count=9 if=gfaydark.s3d of=gfaydark/footer.bin`

    #[test]
    fn it_parses() {
        let mut fixture = File::open("fixtures/gfaydark/footer.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, footer) = Footer::parse(&fixture_data).unwrap();

        assert_eq!(footer.footer_string, b"STEVE");
        assert_eq!(footer.timestamp, 0x5b28ad36);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/footer.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, footer) = Footer::parse(&fixture_data).unwrap();

        assert_eq!(footer.to_bytes(), fixture_data);
    }
}
