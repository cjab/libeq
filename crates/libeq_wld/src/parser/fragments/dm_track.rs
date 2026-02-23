use std::any::Any;

use super::{DmTrackDef2, Fragment, FragmentParser, FragmentRef, StringReference, WResult};

use nom::Parser;
use nom::number::complete::le_u32;

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

    fn parse(input: &[u8]) -> WResult<'_, DmTrack> {
        let (remaining, (name_reference, reference, flags)) =
            (StringReference::parse, FragmentRef::parse, le_u32).parse(input)?;
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
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
            &self.reference.to_bytes()[..],
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

    fn fixture() -> DmTrack {
        DmTrack {
            name_reference: StringReference::new(0),
            reference: FragmentRef::new(0x0278),
            flags: 0,
        }
    }

    #[test]
    fn it_parses() {
        let data = fixture().to_bytes();
        let frag = DmTrack::parse(&data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x0278));
        assert_eq!(frag.flags, 0);
    }

    #[test]
    fn it_serializes() {
        let data = fixture().to_bytes();
        let frag = DmTrack::parse(&data).unwrap().1;

        assert_eq!(frag.to_bytes(), data);
    }
}
