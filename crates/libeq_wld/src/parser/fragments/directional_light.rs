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
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.light_reference.into_bytes()[..],
            &self.flags.into_bytes(),
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
#[derive(Debug, PartialEq)]
pub struct DirectionalLightFlags(u32);

impl DirectionalLightFlags {
    const IS_STATIC: u32 = 0x20;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, raw_flags) = le_u32(input)?;
        Ok((i, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn is_static(&self) -> bool {
        self.0 & Self::IS_STATIC == Self::IS_STATIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        #![allow(overflowing_literals)]
        let data =
            &include_bytes!("../../../fixtures/fragments/wldcom/directional-light-0001-0x2b.frag")
                [..];
        let (remaining, frag) = DirectionalLight::parse(data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(-8));
        assert_eq!(frag.light_reference, FragmentRef::new(1));
        assert_eq!(frag.flags, DirectionalLightFlags(0));
        assert_eq!(frag.normal, (0.26726124, 0.5345225, 0.80178374));
        assert_eq!(frag.num_regions, 1);
        assert_eq!(frag.regions, vec![10]);

        assert_eq!(remaining, vec![]);
    }

    #[test]
    fn it_parses_static() {
        #![allow(overflowing_literals)]
        let data =
            &include_bytes!("../../../fixtures/fragments/wldcom/directional-light-0003-0x2b.frag")
                [..];
        let (remaining, frag) = DirectionalLight::parse(data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(-22));
        assert_eq!(frag.light_reference, FragmentRef::new(3));
        assert_eq!(frag.flags, DirectionalLightFlags(0x20));
        assert_eq!(frag.normal, (0.4558423, 0.5698029, 0.68376344));
        assert_eq!(frag.num_regions, 2);
        assert_eq!(frag.regions, vec![4, 9]);

        assert_eq!(remaining, vec![]);
    }

    #[test]
    fn it_serializes() {
        let data =
            &include_bytes!("../../../fixtures/fragments/wldcom/directional-light-0003-0x2b.frag")
                [..];
        let frag = DirectionalLight::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
