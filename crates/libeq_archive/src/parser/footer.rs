use std::io::Read;

use crate::error::Error;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Footer {
    pub footer_string: [u8; 5],
    pub timestamp: u32,
}

impl Footer {
    pub const FOOTER_STRING: [u8; 5] = *b"STEVE";

    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let mut footer_string = [0u8; 5];
        reader.read_exact(&mut footer_string)?;
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let timestamp = u32::from_be_bytes(buf);

        Ok(Self {
            footer_string,
            timestamp,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [&self.footer_string[..], &self.timestamp.to_be_bytes()].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::{Cursor, Read};

    // Fixture created with:
    // `dd bs=1 skip=2203567 count=9 if=gfaydark.s3d of=gfaydark/footer.bin`

    #[test]
    fn it_reads() {
        let mut fixture = File::open("fixtures/gfaydark/footer.bin").unwrap();
        let footer = Footer::read(&mut fixture).unwrap();

        assert_eq!(&footer.footer_string, b"STEVE");
        assert_eq!(footer.timestamp, 0x36ad285b);
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/footer.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();
        let mut cursor = Cursor::new(&fixture_data);
        let footer = Footer::read(&mut cursor).unwrap();

        assert_eq!(footer.to_bytes(), fixture_data);
    }
}
