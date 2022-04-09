use std::any::Any;

use super::{Fragment, FragmentRef, FragmentType, LightSourceFragment, StringReference};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
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
    const TYPE_NAME: &'static str = "LightSource";

    fn parse(input: &[u8]) -> IResult<&[u8], LightSourceReferenceFragment> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4636-0x1c.frag")[..];
        let frag = LightSourceReferenceFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(1729));
        assert_eq!(frag.flags, 0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4636-0x1c.frag")[..];
        let frag = LightSourceReferenceFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
