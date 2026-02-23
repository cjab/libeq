use std::any::Any;

use super::common::EncodedFilename;
use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::le_u32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// This fragment references one or more texture filenames. So far all known textures
/// reference a single filename.
pub struct BmInfoRtk {
    pub name_reference: StringReference,

    /// Contains the number of texture filenames in this fragment. Again, this appears
    /// to always be 1.
    pub size1: u32,

    pub rtk: u32,

    /// Bitmap filename entries
    pub entries: Vec<EncodedFilename>,
}

impl FragmentParser for BmInfoRtk {
    type T = Self;

    const TYPE_ID: u32 = 0x2c;
    const TYPE_NAME: &'static str = "BmInfoRtk";

    fn parse(input: &[u8]) -> WResult<'_, BmInfoRtk> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, rtk) = le_u32(i)?;
        let (i, size1) = le_u32(i)?;
        // TODO: This is hardcoded to one entry, is this all we need?
        let (remaining, entries) = count(EncodedFilename::parse, (size1 + 1) as usize).parse(i)?;
        Ok((
            remaining,
            BmInfoRtk {
                name_reference,
                size1,
                rtk,
                entries,
            },
        ))
    }
}

impl Fragment for BmInfoRtk {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
            &self.rtk.to_le_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .entries
                .iter()
                .flat_map(|e| e.to_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> BmInfoRtk {
        BmInfoRtk {
            name_reference: StringReference::new(-1),
            rtk: 1,
            size1: 0,
            entries: vec![EncodedFilename {
                name_length: 13, // "TEXTURE1.BMP" + null = 13 bytes
                file_name: "TEXTURE1.BMP".to_string(),
            }],
        }
    }

    #[test]
    fn it_parses() {
        let frag = fixture();
        let data = frag.to_bytes();
        let parsed = BmInfoRtk::parse(&data).unwrap().1;

        assert_eq!(parsed.name_reference, StringReference::new(-1));
        assert_eq!(parsed.rtk, 1);
        assert_eq!(parsed.size1, 0);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].name_length, 13);
        assert_eq!(parsed.entries[0].file_name, "TEXTURE1.BMP".to_string());
    }

    #[test]
    fn it_serializes() {
        let frag = fixture();
        let data = frag.to_bytes();
        let parsed = BmInfoRtk::parse(&data).unwrap().1;

        assert_eq!(parsed.to_bytes(), data);
    }
}
