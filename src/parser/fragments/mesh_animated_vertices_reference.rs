use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, MeshAnimatedVerticesFragment};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [MeshAnimatedVerticesFragment].
///
/// **Type ID:** 0x2f
pub struct MeshAnimatedVerticesReferenceFragment {
    /// The [MeshAnimatedVerticesFragment] reference.
    pub reference: FragmentRef<MeshAnimatedVerticesFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,
}

impl FragmentType for MeshAnimatedVerticesReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2f;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshAnimatedVerticesReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            MeshAnimatedVerticesReferenceFragment { reference, flags },
        ))
    }
}

impl Fragment for MeshAnimatedVerticesReferenceFragment {
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
