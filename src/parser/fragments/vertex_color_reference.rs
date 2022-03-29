use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, MeshAnimatedVerticesFragment};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [VertexColorFragment].
///
/// **Type ID:** 0x33
pub struct VertexColorReferenceFragment {
    /// The [MeshAnimatedVerticesFragment] reference.
    pub reference: FragmentRef<MeshAnimatedVerticesFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentType for VertexColorReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x33;

    fn parse(input: &[u8]) -> IResult<&[u8], VertexColorReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((remaining, VertexColorReferenceFragment { reference, flags }))
    }
}

impl Fragment for VertexColorReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        vec![
            self.reference.serialize().to_le_bytes(),
            self.flags.to_le_bytes(),
        ]
        .iter()
        .flatten()
        .collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
