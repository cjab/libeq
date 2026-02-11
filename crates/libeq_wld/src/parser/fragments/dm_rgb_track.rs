use std::any::Any;

use super::{DmRGBTrackDef, Fragment, FragmentParser, FragmentRef, StringReference, WResult};

use nom::Parser;
use nom::number::complete::le_u32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [DmRGBTrackDef].
///
/// **Type ID:** 0x33
pub struct DmRGBTrack {
    pub name_reference: StringReference,

    /// The [DmRGBTrackDef] reference.
    pub reference: FragmentRef<DmRGBTrackDef>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentParser for DmRGBTrack {
    type T = Self;

    const TYPE_ID: u32 = 0x33;
    const TYPE_NAME: &'static str = "DmRGBTrack";

    fn parse(input: &[u8]) -> WResult<'_, DmRGBTrack> {
        let (remaining, (name_reference, reference, flags)) =
            (StringReference::parse, FragmentRef::parse, le_u32).parse(input)?;
        Ok((
            remaining,
            DmRGBTrack {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for DmRGBTrack {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/objects/0001-0x33.frag")[..];
        let frag = DmRGBTrack::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(1));
        assert_eq!(frag.flags, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/objects/0001-0x33.frag")[..];
        let frag = DmRGBTrack::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
