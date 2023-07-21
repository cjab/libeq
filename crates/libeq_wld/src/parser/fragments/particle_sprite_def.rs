use std::any::Any;

use super::{Fragment, FragmentParser, RenderInfo, RenderMethod, StringReference, WResult};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// PARTICLESPRITEDEF fragment
///
/// **Type ID:** 0x0c
pub struct ParticleSpriteDef {
    pub name_reference: StringReference,

    pub flags: ParticleSpriteDefFlags,

    /// NUMVERTICES %d
    pub num_vertices: u32,

    pub unknown: u32,

    /// CENTEROFFSET %f %f %f
    pub center_offset: Option<(f32, f32, f32)>,

    /// BOUNDINGRADIUS %f
    pub bounding_radius: Option<f32>,

    /// XYZPEN %f %f %f %d
    /// x, y, z (floats) in XYZPEN
    pub vertices: Vec<(f32, f32, f32)>,

    /// RENDERMETHOD ...
    pub render_method: RenderMethod,

    /// RENDERINFO
    pub render_info: RenderInfo,

    /// XYZPEN %f %f %f %d
    /// pen (int) in XYZPEN
    pub pen: Vec<u32>,
}

impl FragmentParser for ParticleSpriteDef {
    type T = Self;

    const TYPE_ID: u32 = 0x0c;
    const TYPE_NAME: &'static str = "ParticleSpriteDef";

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = ParticleSpriteDefFlags::parse(i)?;
        let (i, num_vertices) = le_u32(i)?;
        let (i, unknown) = le_u32(i)?;
        let (i, center_offset) = if flags.has_center_offset() {
            tuple((le_f32, le_f32, le_f32))(i).map(|(i, p3)| (i, Some(p3)))?
        } else {
            (i, None)
        };
        let (i, bounding_radius) = if flags.has_bounding_radius() {
            le_f32(i).map(|(i, b)| (i, Some(b)))?
        } else {
            (i, None)
        };
        let (i, vertices) = count(tuple((le_f32, le_f32, le_f32)), num_vertices as usize)(i)?;
        let (i, render_method) = RenderMethod::parse(i)?;
        let (i, render_info) = RenderInfo::parse(i)?;
        let (i, pen) = count(le_u32, num_vertices as usize)(i)?;

        Ok((
            i,
            Self {
                name_reference,
                flags,
                num_vertices,
                unknown,
                center_offset,
                bounding_radius,
                vertices,
                render_method,
                render_info,
                pen,
            },
        ))
    }
}

impl Fragment for ParticleSpriteDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.num_vertices.to_le_bytes()[..],
            &self.unknown.to_le_bytes()[..],
            &self.center_offset.map_or(vec![], |c| {
                [c.0.to_le_bytes(), c.1.to_le_bytes(), c.2.to_le_bytes()].concat()
            })[..],
            &self
                .bounding_radius
                .map_or(vec![], |b| b.to_le_bytes().to_vec())[..],
            &self
                .vertices
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self.render_method.into_bytes()[..],
            &self.render_info.into_bytes()[..],
            &self
                .pen
                .iter()
                .flat_map(|v| v.to_le_bytes())
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
pub struct ParticleSpriteDefFlags(u32);

impl ParticleSpriteDefFlags {
    const HAS_CENTER_OFFSET: u32 = 0x01;
    const HAS_BOUNDING_RADIUS: u32 = 0x02;

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, raw_flags) = le_u32(input)?;
        Ok((i, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_center_offset(&self) -> bool {
        self.0 & Self::HAS_CENTER_OFFSET == Self::HAS_CENTER_OFFSET
    }

    pub fn has_bounding_radius(&self) -> bool {
        self.0 & Self::HAS_BOUNDING_RADIUS == Self::HAS_BOUNDING_RADIUS
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{MaterialType, RenderInfoFlags};

    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!(
            "../../../fixtures/fragments/wldcom/particle-sprite-0000-0x0c.frag"
        )[..];
        let frag = ParticleSpriteDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-1));
        assert_eq!(frag.flags, ParticleSpriteDefFlags(0x3));
        assert_eq!(frag.flags.has_center_offset(), true);
        assert_eq!(frag.flags.has_bounding_radius(), true);
        assert_eq!(frag.num_vertices, 1);
        assert_eq!(frag.unknown, 0);
        assert_eq!(frag.center_offset, Some((3.0, 4.0, 5.0)));
        assert_eq!(frag.bounding_radius, Some(0.5));
        assert_eq!(frag.vertices[0], (0.0, 0.0, 1.0));

        assert_eq!(
            frag.render_method,
            RenderMethod::UserDefined {
                material_type: MaterialType::Diffuse
            }
        );
        assert_eq!(frag.render_info.flags, RenderInfoFlags::new(2));
        assert_eq!(frag.render_info.flags.has_pen(), false);
        assert_eq!(frag.render_info.flags.has_brightness(), true);
        assert_eq!(frag.render_info.flags.has_scaled_ambient(), false);
        assert_eq!(frag.render_info.flags.has_simple_sprite(), false);
        assert_eq!(frag.render_info.flags.has_uv_info(), false);
        assert_eq!(frag.render_info.flags.is_two_sided(), false);
        assert_eq!(frag.render_info.pen, None);
        assert_eq!(frag.render_info.brightness, Some(1.0));
        assert_eq!(frag.render_info.scaled_ambient, None);
        assert_eq!(frag.render_info.simple_sprite_reference, None);
        assert_eq!(frag.render_info.uv_info, None);
        assert_eq!(frag.render_info.uv_map, None);

        assert_eq!(frag.pen[0], 79);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!(
            "../../../fixtures/fragments/wldcom/particle-sprite-0000-0x0c.frag"
        )[..];
        let frag = ParticleSpriteDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
