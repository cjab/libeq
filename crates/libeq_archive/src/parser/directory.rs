use nom::multi::{count, length_data};
use nom::number::complete::le_u32;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct Directory {
    pub filenames: Vec<String>,
}

impl Directory {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, file_count) = le_u32(input)?;
        let (i, filenames) = count(directory_string, file_count as usize)(i)?;
        Ok((
            i,
            Self {
                filenames: filenames.to_vec(),
            },
        ))
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

fn directory_string(input: &[u8]) -> IResult<&[u8], String> {
    let (i, data) = length_data(le_u32)(input)?;
    Ok((
        i,
        String::from_utf8(Vec::from(data))
            .unwrap()
            // Strings stored in directory are null terminated
            .trim_end_matches('\0')
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Read;

    #[test]
    fn it_parses() {
        let mut fixture = File::open("fixtures/gfaydark/directory.bin").unwrap();
        let mut fixture_data = Vec::new();
        fixture.read_to_end(&mut fixture_data).unwrap();

        let (_, directory) = Directory::parse(&fixture_data).unwrap();

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

        let (_, directory) = Directory::parse(&fixture_data).unwrap();

        assert_eq!(directory.to_bytes(), fixture_data)
    }
}
