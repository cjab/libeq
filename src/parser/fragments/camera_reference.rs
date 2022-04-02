use std::any::Any;

use super::{fragment_ref, CameraFragment, Fragment, FragmentRef, FragmentType, StringReference};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [CameraFragment].
///
/// **Type ID:** 0x09
pub struct CameraReferenceFragment {
    pub name_reference: StringReference,

    /// The [CameraFragment] reference.
    pub reference: FragmentRef<CameraFragment>,

    /// _Unknown_ Seems to always contain 0.
    pub flags: u32,
}

impl FragmentType for CameraReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x09;

    fn parse(input: &[u8]) -> IResult<&[u8], CameraReferenceFragment> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            CameraReferenceFragment {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for CameraReferenceFragment {
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
