use std::any::Any;

use super::{
    Fragment, FragmentParser, FragmentRef, DmTrackDef2, StringReference, WResult,
};

use nom::number::complete::le_u32;
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [DmTrackDef2].
///
/// **Type ID:** 0x2f
pub struct DmTrack {
    pub name_reference: StringReference,

    /// The [DmTrackDef2] reference.
    pub reference: FragmentRef<DmTrackDef2>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentParser for DmTrack {
    type T = Self;

    const TYPE_ID: u32 = 0x2f;
    const TYPE_NAME: &'static str = "DmTrack";

    fn parse(input: &[u8]) -> WResult<DmTrack> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            DmTrack {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for DmTrack {
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark_obj/0632-0x2f.frag")[..];
        let frag = DmTrack::parse(data)
            .unwrap()
            .1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x0278));
        assert_eq!(frag.flags, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark_obj/0632-0x2f.frag")[..];
        let frag = DmTrack::parse(data)
            .unwrap()
            .1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
