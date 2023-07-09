use std::any::Any;

use super::{
    Fragment, FragmentParser, FragmentRef, Light, StringReference, WResult,
};

use nom::number::complete::{le_f32, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [Light].
///
/// **Type ID:** 0x28
pub struct LightInfoFragment {
    pub name_reference: StringReference,

    /// The [Light] reference.
    pub reference: FragmentRef<Light>,

    /// _Unknown_ - Usually contains 256 (0x100).
    pub flags: PointLightFlags,

    /// X component of the light location.
    pub x: f32,

    /// Y component of the light location.
    pub y: f32,

    /// Z component of the light location.
    pub z: f32,

    /// Contains the light radius.
    pub radius: f32,
}

impl FragmentParser for LightInfoFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x28;
    const TYPE_NAME: &'static str = "LightInfo";

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, reference) = FragmentRef::parse(i)?;
        let (i, flags) = PointLightFlags::parse(i)?;
        let (i, x) = le_f32(i)?;
        let (i, y) = le_f32(i)?;
        let (i, z) = le_f32(i)?;
        let (i, radius) = le_f32(i)?;

        Ok((
            i,
            Self {
                name_reference,
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
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.x.to_le_bytes()[..],
            &self.y.to_le_bytes()[..],
            &self.z.to_le_bytes()[..],
            &self.radius.to_le_bytes()[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name_ref(&self) -> &StringReference {
        &self.name_reference
    }

    fn type_id(&self) -> u32 {
        Self::TYPE_ID
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct PointLightFlags(u32);

impl PointLightFlags {
    const IS_STATIC: u32 = 0x20;
    const STATIC_INFLUENCE: u32 = 0x40;
    const HAS_REGIONS: u32 = 0x80;

    fn parse(input: &[u8]) -> WResult<Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn is_static(&self) -> bool {
        self.0 & Self::IS_STATIC == Self::IS_STATIC
    }

    pub fn static_influene(&self) -> bool {
        self.0 & Self::STATIC_INFLUENCE == Self::STATIC_INFLUENCE
    }

    pub fn has_regions(&self) -> bool {
        self.0 & Self::HAS_REGIONS == Self::HAS_REGIONS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/lights/0002-0x28.frag")[..];
        let frag = LightInfoFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(2));
        assert_eq!(frag.flags, PointLightFlags(0x100));
        assert_eq!(frag.x, -1980.6992);
        assert_eq!(frag.y, -2354.9412);
        assert_eq!(frag.z, 31.490416);
        assert_eq!(frag.radius, 300.0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/lights/0002-0x28.frag")[..];
        let frag = LightInfoFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
