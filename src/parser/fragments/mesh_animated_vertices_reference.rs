use std::any::Any;

use super::{Fragment, FragmentRef, FragmentParser, MeshAnimatedVerticesFragment, StringReference};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [MeshAnimatedVerticesFragment].
///
/// **Type ID:** 0x2f
pub struct MeshAnimatedVerticesReferenceFragment {
    pub name_reference: StringReference,

    /// The [MeshAnimatedVerticesFragment] reference.
    pub reference: FragmentRef<MeshAnimatedVerticesFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentParser for MeshAnimatedVerticesReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2f;
    const TYPE_NAME: &'static str = "MeshAnimatedVerticesReference";

    fn parse(input: &[u8]) -> IResult<&[u8], MeshAnimatedVerticesReferenceFragment> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            MeshAnimatedVerticesReferenceFragment {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for MeshAnimatedVerticesReferenceFragment {
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark_obj/0632-0x2f.frag")[..];
        let frag = MeshAnimatedVerticesReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x0278));
        assert_eq!(frag.flags, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark_obj/0632-0x2f.frag")[..];
        let frag = MeshAnimatedVerticesReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(&frag.serialize()[..], data);
    }
}