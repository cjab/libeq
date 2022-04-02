use std::any::Any;

use super::{
    fragment_ref, Fragment, FragmentRef, FragmentType, LightSourceFragment, StringReference,
};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [LightSourceFragment].
///
/// **Type ID:** 0x1c
pub struct LightSourceReferenceFragment {
    pub name_reference: StringReference,

    /// The [LightSourceFragment] reference.
    pub reference: FragmentRef<LightSourceFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentType for LightSourceReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x1c;

    fn parse(input: &[u8]) -> IResult<&[u8], LightSourceReferenceFragment> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            LightSourceReferenceFragment {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for LightSourceReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.reference.serialize()[..],
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
}
