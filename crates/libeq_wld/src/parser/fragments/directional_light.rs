use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, Light, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// DIRECTIONALLIGHT fragment
///
/// **Type ID:** 0x2b
pub struct DirectionalLight {
    /// TAG "%s"
    pub name_reference: StringReference,

    /// LIGHT "%s"
    pub light_reference: FragmentRef<Light>,

    /// STATIC
    pub flags: DirectionalLightFlags,

    /// NORMAL %f %f %f
    /// vector is normalized when compressed
    pub normal: (f32, f32, f32),

    /// NUMREGIONS %d
    pub num_regions: u32,

    /// REGIONS %d ...%d
    pub regions: Vec<u32>,
}

impl FragmentParser for DirectionalLight {
    type T = Self;

    const TYPE_ID: u32 = 0x2b;
    const TYPE_NAME: &'static str = "DirectionalLight";

    fn parse(input: &[u8]) -> WResult<'_, DirectionalLight> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, light_reference) = FragmentRef::parse(i)?;
        let (i, flags) = DirectionalLightFlags::parse(i)?;
        let (i, normal) = (le_f32, le_f32, le_f32).parse(i)?;
        let (i, num_regions) = le_u32(i)?;
        let (remainder, regions) = count(le_u32, num_regions as usize).parse(i)?;

        Ok((
            remainder,
            DirectionalLight {
                name_reference,
                light_reference,
                flags,
                normal,
                num_regions,
                regions,
            },
        ))
    }
}

impl Fragment for DirectionalLight {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
            &self.light_reference.to_bytes()[..],
            &self.flags.to_bytes(),
            &self.normal.0.to_le_bytes()[..],
            &self.normal.1.to_le_bytes()[..],
            &self.normal.2.to_le_bytes()[..],
            &self.num_regions.to_le_bytes()[..],
            &self
                .regions
                .iter()
                .flat_map(|r| r.to_le_bytes())
                .collect::<Vec<_>>()[..],
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
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct DirectionalLightFlags(u32);

impl From<DirectionalLightFlags> for u32 {
    fn from(flags: DirectionalLightFlags) -> u32 {
        flags.0
    }
}

impl DirectionalLightFlags {
    const IS_STATIC: u32 = 0x20;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, raw_flags) = le_u32(input)?;
        Ok((i, Self(raw_flags)))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn is_static(&self) -> bool {
        self.0 & Self::IS_STATIC == Self::IS_STATIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> DirectionalLight {
        DirectionalLight {
            name_reference: StringReference::new(-8),
            light_reference: FragmentRef::new(1),
            flags: DirectionalLightFlags(0),
            normal: (0.5, 0.5, 0.5),
            num_regions: 1,
            regions: vec![10],
        }
    }

    fn fixture_static() -> DirectionalLight {
        DirectionalLight {
            name_reference: StringReference::new(-22),
            light_reference: FragmentRef::new(3),
            flags: DirectionalLightFlags(0x20),
            normal: (0.25, 0.5, 0.75),
            num_regions: 2,
            regions: vec![4, 9],
        }
    }

    #[test]
    fn it_parses() {
        let data = fixture().to_bytes();
        let (remaining, frag) = DirectionalLight::parse(&data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(-8));
        assert_eq!(frag.light_reference, FragmentRef::new(1));
        assert_eq!(frag.flags, DirectionalLightFlags(0));
        assert_eq!(frag.normal, (0.5, 0.5, 0.5));
        assert_eq!(frag.num_regions, 1);
        assert_eq!(frag.regions, vec![10]);
        assert!(remaining.is_empty());
    }

    #[test]
    fn it_parses_static() {
        let data = fixture_static().to_bytes();
        let (remaining, frag) = DirectionalLight::parse(&data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(-22));
        assert_eq!(frag.light_reference, FragmentRef::new(3));
        assert_eq!(frag.flags, DirectionalLightFlags(0x20));
        assert_eq!(frag.normal, (0.25, 0.5, 0.75));
        assert_eq!(frag.num_regions, 2);
        assert_eq!(frag.regions, vec![4, 9]);
        assert!(remaining.is_empty());
    }

    #[test]
    fn it_serializes() {
        let data = fixture_static().to_bytes();
        let frag = DirectionalLight::parse(&data).unwrap().1;

        assert_eq!(frag.to_bytes(), data);
    }
}
