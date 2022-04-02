use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, PolygonAnimationFragment};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [PolygonAnimationFragment].
///
/// **Type ID:** 0x18
pub struct PolygonAnimationReferenceFragment {
    /// The [PolygonAnimationFragment] reference.
    pub reference: FragmentRef<PolygonAnimationFragment>,

    /// _Unknown_
    /// * bit 0 - If set `params1` exists.
    pub flags: u32,

    /// _Unknown_
    pub params1: f32,
}

impl FragmentType for PolygonAnimationReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x18;

    fn parse(input: &[u8]) -> IResult<&[u8], PolygonAnimationReferenceFragment> {
        let (remaining, (reference, flags, params1)) =
            tuple((fragment_ref, le_u32, le_f32))(input)?;
        Ok((
            remaining,
            PolygonAnimationReferenceFragment {
                reference,
                flags,
                params1,
            },
        ))
    }
}

impl Fragment for PolygonAnimationReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.reference.serialize().to_le_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.params1.to_le_bytes()[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
