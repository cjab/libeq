use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, MeshFragment, StringHash};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [MeshFragment] fragment.
///
/// **Type ID:** 0x2d
pub struct MeshReferenceFragment {
    /// The [MeshFragment] reference.
    pub reference: FragmentRef<MeshFragment>,

    /// _Unknown_ - Apparently must be zero.
    pub params: u32,
}

impl FragmentType for MeshReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2d;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshReferenceFragment> {
        let (remaining, (reference, params)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((remaining, MeshReferenceFragment { reference, params }))
    }
}

impl Fragment for MeshReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.reference.serialize().to_le_bytes()[..],
            &self.params.to_le_bytes()[..],
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
