use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, LightSourceReferenceFragment};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [LightSourceReferenceFragment].
///
/// **Type ID:** 0x28
pub struct LightInfoFragment {
    /// The [LightSourceReferenceFragment] reference.
    pub reference: FragmentRef<LightSourceReferenceFragment>,

    /// _Unknown_ - Usually contains 256 (0x100).
    pub flags: u32,

    /// X component of the light location.
    pub x: f32,

    /// Y component of the light location.
    pub y: f32,

    /// Z component of the light location.
    pub z: f32,

    /// Contains the light radius.
    pub radius: f32,
}

impl FragmentType for LightInfoFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x28;

    fn parse(input: &[u8]) -> IResult<&[u8], LightInfoFragment> {
        let (remaining, (reference, flags, x, y, z, radius)) =
            tuple((fragment_ref, le_u32, le_f32, le_f32, le_f32, le_f32))(input)?;
        Ok((
            remaining,
            LightInfoFragment {
                reference,
                flags,
                x,
                y,
                z,
                radius,
            },
        ))
    }
}

impl Fragment for LightInfoFragment {
    fn serialize(&self) -> Vec<u8> {
        vec![
            self.reference.serialize().to_le_bytes(),
            self.flags.to_le_bytes(),
            self.x.to_le_bytes(),
            self.y.to_le_bytes(),
            self.z.to_le_bytes(),
            self.radius.to_le_bytes(),
        ]
        .iter()
        .flatten()
        .collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
