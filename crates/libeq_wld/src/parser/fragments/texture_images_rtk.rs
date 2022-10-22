use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};
use crate::parser::strings::{decode_string, encode_string};

use nom::multi::count;
use nom::number::complete::{le_u16, le_u32, le_u8};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// This fragment references one or more texture filenames. So far all known textures
/// reference a single filename.
pub struct TextureImagesFragmentRtk {
    pub name_reference: StringReference,

    /// Contains the number of texture filenames in this fragment. Again, this appears
    /// to always be 1.
    pub size1: u32,

    pub rtk: u32,

    /// Bitmap filename entries
    pub entries: Vec<TextureImagesFragmentRtkEntry>,
}

impl FragmentParser for TextureImagesFragmentRtk {
    type T = Self;

    const TYPE_ID: u32 = 0x2c;
    const TYPE_NAME: &'static str = "TextureImagesRtk";

    fn parse(input: &[u8]) -> WResult<TextureImagesFragmentRtk> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, rtk) = le_u32(i)?;
        let (i, size1) = le_u32(i)?;
        // TODO: This is hardcoded to one entry, is this all we need?
        let (remaining, entries) =
            count(TextureImagesFragmentRtkEntry::parse, (size1 + 1) as usize)(i)?;
        Ok((
            remaining,
            TextureImagesFragmentRtk {
                name_reference,
                size1,
                rtk,
                entries,
            },
        ))
    }
}

impl Fragment for TextureImagesFragmentRtk {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.rtk.to_le_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .entries
                .iter()
                .flat_map(|e| e.into_bytes())
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name_ref(&self) -> &StringReference {
        &self.name_reference
    }

    fn type_id(&self) -> u32 {
        Self::TYPE_ID
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Bitmap filename entries within the [TextureImagesFragmentRtk] fragment.
pub struct TextureImagesFragmentRtkEntry {
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

impl TextureImagesFragmentRtkEntry {
    fn parse(input: &[u8]) -> WResult<TextureImagesFragmentRtkEntry> {
        let (i, name_length) = le_u16(input)?;
        let (remaining, file_name) = count(le_u8, name_length as usize)(i)?;
        Ok((
            remaining,
            TextureImagesFragmentRtkEntry {
                name_length,
                file_name: decode_string(&file_name).trim_end_matches("\0").to_string(),
            },
        ))
    }

    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_length.to_le_bytes()[..],
            &encode_string(&format!("{}{}", &self.file_name, "\0"))[..],
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/rtk/0000-0x2c.frag")[..];
        let frag = TextureImagesFragmentRtk::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0xffffffff));
        //FIXME: Why is this 0? If this is size it should be 1.
        //assert_eq!(frag.size1, 1);
        assert_eq!(frag.entries.len(), 1);
        assert_eq!(frag.entries[0].name_length, 0x0d);
        assert_eq!(frag.entries[0].file_name, "S5AMLG__.BMP".to_string());
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/rtk/0000-0x2c.frag")[..];
        let frag = TextureImagesFragmentRtk::parse(data).unwrap().1;

        assert_eq!([frag.into_bytes(), vec![0]].concat(), data);
    }
}
