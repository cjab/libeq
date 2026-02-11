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
/// FRAME and BMINFO fragments.
///
/// This fragment associates a name to one or more texture filenames.
/// Starting with Luclin (new wld version), zones appear to implement layered terrain
/// details using multiple entries.
///
/// WLDCOM decompresses a single entry into FRAME and multiple entries into BMINFO.
/// The parameter order is also swapped.
/// FRAME "FileName1.bmp" "Name"
/// BMINFO "Name" "FileName1.bmp" "FileName2.dds"
///
/// **Type ID:** 0x03
pub struct BmInfo {
    pub name_reference: StringReference,

    /// Contains the number of texture filenames in this fragment minus 1.
    /// For example, an `entry_count` of 5 corresponds to 6 `entries`.
    pub entry_count: u32,

    /// Bitmap filename entries
    /// FRAME %s %s
    /// BMINFO %s %s...
    pub entries: Vec<EncodedFilename>,
}

impl FragmentParser for BmInfo {
    type T = Self;

    const TYPE_ID: u32 = 0x03;
    const TYPE_NAME: &'static str = "BmInfo";

    fn parse(input: &[u8]) -> WResult<'_, BmInfo> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, entry_count) = le_u32(i)?;
        let (remaining, entries) =
            count(EncodedFilename::parse, (entry_count + 1) as usize).parse(i)?;
        Ok((
            remaining,
            BmInfo {
                name_reference,
                entry_count,
                entries,
            },
        ))
    }
}

impl Fragment for BmInfo {
    fn into_bytes(&self) -> Vec<u8> {
        let entry_count: u32 = (self.entries.len() - 1).try_into().unwrap();
        let bytes = [
            &self.name_reference.into_bytes()[..],
            &entry_count.to_le_bytes()[..],
            &self
                .entries
                .iter()
                .flat_map(|e| e.into_bytes())
                .collect::<Vec<_>>()[..],
        ]
        .concat();

        let padding_size = (3 - bytes.len() % 4) % 4;
        let padding: Vec<u8> = vec![0; padding_size];

        [&bytes[..], &padding[..]].concat()
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0001-0x03.frag")[..];
        let frag = BmInfo::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0xffffffff));
        assert_eq!(frag.entry_count, 0);
        assert_eq!(frag.entries.len(), 1);
        assert_eq!(frag.entries[0].name_length, 0x0b);
        assert_eq!(frag.entries[0].file_name, "SGRASS.BMP".to_string());
    }

    #[test]
    fn it_parses_multiple_entries() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/twilight/0000-0x03.frag")[..];
        let frag = BmInfo::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0xffffffff));
        assert_eq!(frag.entry_count, 7);
        assert_eq!(frag.entries.len(), 8);
        assert_eq!(frag.entries[7].name_length, 21);
        assert_eq!(
            frag.entries[7].file_name,
            "6, 5, 0, SAND02A.DDS".to_string()
        );
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0029-0x03.frag")[..];
        let frag = BmInfo::parse(data).unwrap().1;

        assert_eq!([frag.into_bytes(), vec![0]].concat(), data);
    }

    #[test]
    fn it_serializes_with_multiple_entries_and_padding() {
        let data = &include_bytes!("../../../fixtures/fragments/twilight/0000-0x03.frag")[..];
        let frag = BmInfo::parse(data).unwrap().1;

        assert_eq!([frag.into_bytes(), vec![0]].concat(), data);
    }
}
