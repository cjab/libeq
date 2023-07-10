use std::any::Any;

use super::common::EncodedFilename;
use super::{Fragment, FragmentParser, StringReference, WResult};

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

    fn parse(input: &[u8]) -> WResult<BmInfoRtk> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, rtk) = le_u32(i)?;
        let (i, size1) = le_u32(i)?;
        // TODO: This is hardcoded to one entry, is this all we need?
        let (remaining, entries) = count(EncodedFilename::parse, (size1 + 1) as usize)(i)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/rtk/0000-0x2c.frag")[..];
        let frag = BmInfoRtk::parse(data).unwrap().1;

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
        let frag = BmInfoRtk::parse(data).unwrap().1;

        assert_eq!([frag.into_bytes(), vec![0]].concat(), data);
    }
}
