use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, MeshFragment, StringReference};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [MeshFragment] fragment.
///
/// **Type ID:** 0x2d
pub struct MeshReferenceFragment {
    pub name_reference: StringReference,

    /// The [MeshFragment] reference.
    pub reference: FragmentRef<MeshFragment>,

    /// _Unknown_ - Apparently must be zero.
    pub params: u32,
}

impl FragmentType for MeshReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2d;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshReferenceFragment> {
        let (remaining, (name_reference, reference, params)) =
            tuple((StringReference::parse, fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            MeshReferenceFragment {
                name_reference,
                reference,
                params,
            },
        ))
    }
}

impl Fragment for MeshReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.reference.serialize()[..],
            &self.params.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0012-0x2d.frag")[..];
        let frag = MeshReferenceFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(6));
        assert_eq!(frag.params, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0012-0x2d.frag")[..];
        let frag = MeshReferenceFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
