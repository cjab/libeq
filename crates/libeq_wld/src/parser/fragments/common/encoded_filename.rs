use super::WResult;
use crate::parser::strings::{decode_string, encode_string};

use nom::multi::count;
use nom::number::complete::{le_u16, le_u8};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Bitmap filename entries within [BmInfo] and [PaletteFileFragment].
pub struct EncodedFilename {
    /// The length of the filename in bytes.
    pub name_length: u16,

    /// The encoded filename. See [string hash encoding].
    ///
    /// The client apparently looks for certain filenames and substitutes built-in
    /// textures in their place. When using an animated fire texture where the names
    /// are fire1.bmp, fire2.bmp, fire3.bmp and fire4.bmp, respectively, the client always
    /// uses its built-in fire textures instead. This only happens when the textures are
    /// used by a placeable object and not when the textures are in the main zone file.
    /// It is unknown whether the substitution depends on the presence and exact order
    /// of all four textures.
    pub file_name: String,
}

impl EncodedFilename {
    pub fn parse(input: &[u8]) -> WResult<EncodedFilename> {
        let (i, name_length) = le_u16(input)?;
        let (remaining, file_name) = count(le_u8, name_length as usize)(i)?;
        Ok((
            remaining,
            EncodedFilename {
                name_length,
                file_name: decode_string(&file_name).trim_end_matches("\0").to_string(),
            },
        ))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_length.to_le_bytes()[..],
            &encode_string(&format!("{}{}", &self.file_name, "\0"))[..],
        ]
        .concat()
    }
}
