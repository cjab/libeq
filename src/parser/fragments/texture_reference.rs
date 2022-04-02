use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, StringReference, TextureFragment};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [TextureFragment] fragment.
///
/// **Type ID:** 0x05
pub struct TextureReferenceFragment {
    pub name_reference: StringReference,

    /// The [TextureFragment] reference.
    pub reference: FragmentRef<TextureFragment>,

    /// _Unknown_ - Seems to always contain 0x50.
    pub flags: u32,
}

impl FragmentType for TextureReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x05;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureReferenceFragment> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            TextureReferenceFragment {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for TextureReferenceFragment {
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
