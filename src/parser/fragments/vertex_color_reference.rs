use std::any::Any;

use super::{Fragment, FragmentRef, FragmentParser, MeshAnimatedVerticesFragment, StringReference};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
/// A reference to a [VertexColorFragment].
///
/// **Type ID:** 0x33
pub struct VertexColorReferenceFragment {
    pub name_reference: StringReference,

    /// The [MeshAnimatedVerticesFragment] reference.
    pub reference: FragmentRef<MeshAnimatedVerticesFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentParser for VertexColorReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x33;
    const TYPE_NAME: &'static str = "VertexColorReference";

    fn parse(input: &[u8]) -> IResult<&[u8], VertexColorReferenceFragment> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            VertexColorReferenceFragment {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for VertexColorReferenceFragment {
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
        let data = &include_bytes!("../../../fixtures/fragments/objects/0001-0x33.frag")[..];
        let frag = VertexColorReferenceFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(1));
        assert_eq!(frag.flags, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/objects/0001-0x33.frag")[..];
        let frag = VertexColorReferenceFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
