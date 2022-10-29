use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// SPHERELISTDEFINITION fragment
///
/// **Type ID:** 0x19
pub struct SphereListDefFragment {
    pub name_reference: StringReference,

    pub flags: SphereListDefFlags,

    /// NUMSPHERES %d
    pub num_spheres: u32,

    /// BOUNDINGRADIUS %f
    pub bounding_radius: f32,

    /// SCALEFACTOR %f
    pub scale_factor: Option<f32>,

    /// SPHERE %f %f %f %f
    pub spheres: Vec<(f32, f32, f32, f32)>,
}

impl FragmentParser for SphereListDefFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x19;
    const TYPE_NAME: &'static str = "SphereListDef";

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = SphereListDefFlags::parse(i)?;
        let (i, num_spheres) = le_u32(i)?;
        let (i, bounding_radius) = le_f32(i)?;
        let (i, scale_factor) = if flags.has_scale_factor() {
            le_f32(i).map(|(i, s)| (i, Some(s)))?
        } else {
            (i, None)
        };
        let (i, spheres) = count(
            tuple((le_f32, le_f32, le_f32, le_f32)),
            num_spheres as usize,
        )(i)?;

        Ok((
            i,
            Self {
                name_reference,
                flags,
                num_spheres,
                bounding_radius,
                scale_factor,
                spheres,
            },
        ))
    }
}

impl Fragment for SphereListDefFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.num_spheres.to_le_bytes()[..],
            &self.bounding_radius.to_le_bytes()[..],
            &self
                .scale_factor
                .map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
            &self
                .spheres
                .iter()
                .flat_map(|v| {
                    [
                        v.0.to_le_bytes(),
                        v.1.to_le_bytes(),
                        v.2.to_le_bytes(),
                        v.3.to_le_bytes(),
                    ]
                    .concat()
                })
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
pub struct SphereListDefFlags(u32);

impl SphereListDefFlags {
    const HAS_SCALE_FACTOR: u32 = 0x01;

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, raw_flags) = le_u32(input)?;
        Ok((i, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_scale_factor(&self) -> bool {
        self.0 & Self::HAS_SCALE_FACTOR == Self::HAS_SCALE_FACTOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/tanarus-equip/2907-0x19.frag")[..];
        let frag = SphereListDefFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-11175));
        assert_eq!(frag.flags, SphereListDefFlags(0x1));
        assert_eq!(frag.num_spheres, 2);
        assert_eq!(frag.bounding_radius, 4.980589);
        assert_eq!(frag.scale_factor, Some(1.0));
        assert_eq!(frag.spheres[0], (4.545429, 0.575291, 0.000907, 0.398899));
        assert_eq!(frag.spheres[1], (4.545429, -0.571938, 0.000907, 0.398899));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/tanarus-equip/2907-0x19.frag")[..];
        let frag = SphereListDefFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
