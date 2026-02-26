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

    pub fn to_bytes(self) -> Vec<u8> {
        [&self.footer_string[..], &self.timestamp.to_be_bytes()].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> Footer {
        Footer {
            footer_string: *b"STEVE",
            timestamp: 1000000000,
        }
    }

    #[test]
    fn it_reads() {
        let data = fixture().to_bytes();
        let footer = Footer::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(footer.footer_string, *b"STEVE");
        assert_eq!(footer.timestamp, 1000000000);
    }

    #[test]
    fn it_serializes() {
        let data = fixture().to_bytes();
        let footer = Footer::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(footer.to_bytes(), data);
    }
}
