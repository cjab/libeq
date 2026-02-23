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
    use std::io::Cursor;

    fn fixture() -> Directory {
        Directory {
            filenames: vec![
                "texture.bmp".to_string(),
                "model.wld".to_string(),
                "sound.wav".to_string(),
            ],
        }
    }

    #[test]
    fn it_reads() {
        let data = fixture().to_bytes();
        let dir = Directory::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(dir.filenames.len(), 3);
        assert_eq!(dir.filenames[0], "texture.bmp");
        assert_eq!(dir.filenames[1], "model.wld");
        assert_eq!(dir.filenames[2], "sound.wav");
    }

    #[test]
    fn it_serializes() {
        let data = fixture().to_bytes();
        let dir = Directory::read(&mut Cursor::new(&data)).unwrap();

        assert_eq!(dir.to_bytes(), data);
    }
}
