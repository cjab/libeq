use std::any::Any;

use super::{decode_string, encode_string};
use super::{Fragment, FragmentType, StringReference};

use nom::multi::count;
use nom::number::complete::{le_u16, le_u32, le_u8};
use nom::IResult;

#[derive(Debug, PartialEq)]
/// This fragment references one or more texture filenames. So far all known textures
/// reference a single filename.
pub struct TextureImagesFragment {
    pub name_reference: StringReference,

    /// Contains the number of texture filenames in this fragment. Again, this appears
    /// to always be 1.
    pub size1: u32,

    /// Bitmap filename entries
    pub entries: Vec<TextureImagesFragmentEntry>,
}

impl FragmentType for TextureImagesFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x03;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureImagesFragment> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, size1) = le_u32(i)?;
        // TODO: This is hardcoded to one entry, is this all we need?
        let (remaining, entries) = count(TextureImagesFragmentEntry::parse, 1 as usize)(i)?;
        Ok((
            remaining,
            TextureImagesFragment {
                name_reference,
                size1,
                entries,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
/// Bitmap filename entries within the [TextureImagesFragment] fragment.
pub struct TextureImagesFragmentEntry {
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

impl TextureImagesFragmentEntry {
    fn parse(input: &[u8]) -> IResult<&[u8], TextureImagesFragmentEntry> {
        let (i, name_length) = le_u16(input)?;
        let (remaining, file_name) = count(le_u8, name_length as usize)(i)?;
        Ok((
            remaining,
            TextureImagesFragmentEntry {
                name_length,
                file_name: decode_string(&file_name),
            },
        ))
    }

    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_length.to_le_bytes()[..],
            &encode_string(&self.file_name)[..],
        ]
        .concat()
    }
}

impl Fragment for TextureImagesFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .entries
                .iter()
                .flat_map(|e| e.serialize())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0001-0x03.frag")[..];
        let frag = TextureImagesFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0xffffffff));
        //FIXME: Why is this 0? If this is size it should be 1.
        //assert_eq!(frag.size1, 1);
        assert_eq!(frag.entries.len(), 1);
        assert_eq!(frag.entries[0].name_length, 0x0b);
        assert_eq!(frag.entries[0].file_name, "SGRASS.BMP".to_string());
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0029-0x03.frag")[..];
        let frag = TextureImagesFragment::parse(data).unwrap().1;

        println!("SIZE: {}", frag.size1);

        assert_eq!(&frag.serialize()[..], data);
    }
}
