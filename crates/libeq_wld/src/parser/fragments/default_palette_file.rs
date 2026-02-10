use std::any::Any;

use super::common::EncodedFilename;
use super::{Fragment, FragmentParser, StringReference, WResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// DEFAULTPALETTEFILE fragment
///
/// **Type ID:** 0x01
pub struct DefaultPaletteFile {
    pub name_reference: StringReference,

    pub entry: EncodedFilename,
}

impl FragmentParser for DefaultPaletteFile {
    type T = Self;

    const TYPE_ID: u32 = 0x01;
    const TYPE_NAME: &'static str = "DefaultPaletteFile";

    fn parse(input: &[u8]) -> WResult<'_, DefaultPaletteFile> {
        let name_reference = StringReference::new(0);
        let (remainder, entry) = EncodedFilename::parse(input)?;
        Ok((
            remainder,
            DefaultPaletteFile {
                name_reference,
                entry,
            },
        ))
    }
}

impl Fragment for DefaultPaletteFile {
    fn into_bytes(&self) -> Vec<u8> {
        let entry = &self.entry.into_bytes()[..];
        let padding_size = (4 - entry.len() % 4) % 4;
        let padding: Vec<u8> = vec![0; padding_size];

        [entry, &padding[..]].concat()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        #![allow(overflowing_literals)]
        let data =
            &include_bytes!("../../../fixtures/fragments/tanarus-thecity/0000-0x01.frag")[..];
        let frag = DefaultPaletteFile::parse(data).unwrap().1;

        assert_eq!(
            frag.entry,
            EncodedFilename {
                name_length: 12,
                file_name: String::from("palette.bmp")
            }
        );
    }

    #[test]
    fn it_serializes() {
        let data =
            &include_bytes!("../../../fixtures/fragments/tanarus-thecity/0000-0x01.frag")[..];
        let frag = DefaultPaletteFile::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
