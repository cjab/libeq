use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, StringHash};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [TwoDimensionalObjectReferenceFragment].
///
/// **Type ID:** 0x07
pub struct TwoDimensionalObjectReferenceFragment {
    /// The [TwoDimensionalObjectFragment] reference.
    pub reference: FragmentRef<TwoDimensionalObjectReferenceFragment>,

    /// _Unknown_ Seems to always contain 0.
    pub flags: u32,
}

impl FragmentType for TwoDimensionalObjectReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x07;

    fn parse(input: &[u8]) -> IResult<&[u8], TwoDimensionalObjectReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            TwoDimensionalObjectReferenceFragment { reference, flags },
        ))
    }
}

impl Fragment for TwoDimensionalObjectReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
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
