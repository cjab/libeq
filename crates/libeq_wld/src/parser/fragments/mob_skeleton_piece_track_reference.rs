use std::any::Any;

use super::{
    Fragment, FragmentParser, FragmentRef, MobSkeletonPieceTrackFragment, StringReference,
};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [MobSkeletonPieceTrackFragment].
///
/// **Type ID:** 0x13
pub struct MobSkeletonPieceTrackReferenceFragment {
    pub name_reference: StringReference,

    /// The [MobSkeletonPieceTrackFragment] reference.
    pub reference: FragmentRef<MobSkeletonPieceTrackFragment>,

    /// Most flags are _unknown_
    /// * bit 0 - If set `params1` exists.
    /// * bit 2 - Usually set.
    pub flags: u32,

    /// _Unknown_
    pub params1: Option<u32>,
}

impl FragmentParser for MobSkeletonPieceTrackReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x13;
    const TYPE_NAME: &'static str = "MobSkeletonPieceTrackReference";

    fn parse(input: &[u8]) -> IResult<&[u8], MobSkeletonPieceTrackReferenceFragment> {
        let (i, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;

        let (remaining, params1) = if flags & 0x01 == 0x01 {
            le_u32(i).map(|(i, params1)| (i, Some(params1)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            MobSkeletonPieceTrackReferenceFragment {
                name_reference,
                reference,
                flags,
                params1,
            },
        ))
    }
}

impl Fragment for MobSkeletonPieceTrackReferenceFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.params1.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0007-0x13.frag")[..];
        let frag = MobSkeletonPieceTrackReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(frag.name_reference, StringReference::new(-75));
        assert_eq!(frag.reference, FragmentRef::new(7));
        assert_eq!(frag.flags, 0x0);
        assert_eq!(frag.params1, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0007-0x13.frag")[..];
        let frag = MobSkeletonPieceTrackReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
