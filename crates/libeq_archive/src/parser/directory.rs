use std::io::Read;

use crate::error::Error;

const MAX_FILENAME_LENGTH: u32 = 1024; // 1KB filename

#[derive(Debug, PartialEq)]
pub struct Directory {
    pub filenames: Vec<String>,
}

impl Directory {
    pub fn read(reader: &mut impl Read) -> Result<Self, Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let file_count = u32::from_le_bytes(buf);
        let filenames = (0..file_count)
            .map(|_| directory_string(reader))
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self { filenames })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            &(self.filenames.len() as u32).to_le_bytes()[..],
            &self
                .filenames
                .iter()
                .flat_map(|f| {
                    let null_terminated = format!("{}\0", f);
                    [
                        &(null_terminated.len() as u32).to_le_bytes()[..],
                        &null_terminated.into_bytes(),
                    ]
                    .concat()
                })
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }
}

fn directory_string(reader: &mut impl Read) -> Result<String, Error> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let length = u32::from_le_bytes(buf);
    if length > MAX_FILENAME_LENGTH {
        return Err(Error::CorruptArchive(format!(
            "directory contains filename of length {}, this exceeds the max of {}",
            length, MAX_FILENAME_LENGTH
        )));
    }
    let mut string_data = vec![0u8; length as usize];
    reader.read_exact(&mut string_data)?;

    Ok(String::from_utf8(string_data)
        .map_err(|_| Error::CorruptArchive("Invalid utf8 data in file directory".into()))?
        // Strings stored in directory are null terminated
        .trim_end_matches('\0')
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::{Cursor, Read};

    #[test]
    fn it_reads() {
        let mut fixture = File::open("fixtures/gfaydark/directory.bin").unwrap();
        let directory = Directory::read(&mut fixture).unwrap();

        assert_eq!(
            directory,
            Directory {
                filenames: vec![
                    "palette.bmp".to_string(),
                    "sfrg4.bmp".to_string(),
                    "sfrg5.bmp".to_string(),
                    "1dirtfloor.bmp".to_string(),
                    "grassrock.bmp".to_string(),
                    "sfrg6.bmp".to_string(),
                    "sfrg7.bmp".to_string(),
                    "sfrg8.bmp".to_string(),
                    "sfrg9.bmp".to_string(),
                    "sfrg10.bmp".to_string(),
                    "spath.bmp".to_string(),
                    "spath45.bmp".to_string(),
                    "spathend.bmp".to_string(),
                    "spathlh.bmp".to_string(),
                    "spathrh.bmp".to_string(),
                    "spatht.bmp".to_string(),
                    "spathtol.bmp".to_string(),
                    "spathtor.bmp".to_string(),
                    "spathy1.bmp".to_string(),
                    "rockw2.bmp".to_string(),
                    "xgrass1.bmp".to_string(),
                    "fewall01.bmp".to_string(),
                    "grastran.bmp".to_string(),
                    "citystone.bmp".to_string(),
                    "citywall.bmp".to_string(),
                    "fayfloor.bmp".to_string(),
                    "nekpine.bmp".to_string(),
                    "brailing1.bmp".to_string(),
                    "fayroof1.bmp".to_string(),
                    "faywall1.bmp".to_string(),
                    "kbark.bmp".to_string(),
                    "kbarkd.bmp".to_string(),
                    "kbarkt.bmp".to_string(),
                    "plank3.bmp".to_string(),
                    "ub5.bmp".to_string(),
                    "kbarkd1.bmp".to_string(),
                    "sgrass.bmp".to_string(),
                    "gfaydark.wld".to_string(),
                    "objects.wld".to_string(),
                    "lights.wld".to_string(),
                ]
            }
        );
    }

    #[test]
    fn it_serializes() {
        let mut fixture = File::open("fixtures/gfaydark/directory.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();
        let mut cursor = Cursor::new(&fixture_data);
        let directory = Directory::read(&mut cursor).unwrap();

        assert_eq!(directory.to_bytes(), fixture_data)
    }
}
