use std::any::Any;

use super::{
    fragment_ref, Fragment, FragmentRef, FragmentType, MobSkeletonPieceTrackFragment, StringHash,
};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [MobSkeletonPieceTrackFragment].
///
/// **Type ID:** 0x13
pub struct MobSkeletonPieceTrackReferenceFragment {
    /// The [MobSkeletonPieceTrackFragment] reference.
    pub reference: FragmentRef<MobSkeletonPieceTrackFragment>,

    /// Most flags are _unknown_
    /// * bit 0 - If set `params1` exists.
    /// * bit 2 - Usually set.
    pub flags: u32,

    /// _Unknown_
    pub params1: Option<u32>,
}

impl FragmentType for MobSkeletonPieceTrackReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x13;

    fn parse(input: &[u8]) -> IResult<&[u8], MobSkeletonPieceTrackReferenceFragment> {
        let (i, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;

        let (remaining, params1) = if flags & 0x01 == 0x01 {
            le_u32(i).map(|(i, params1)| (i, Some(params1)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            MobSkeletonPieceTrackReferenceFragment {
                reference,
                flags,
                params1,
            },
        ))
    }
}

impl Fragment for MobSkeletonPieceTrackReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.reference.serialize().to_le_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.params1.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self, string_hash: &StringHash) -> String {
        String::new()
    }
}
