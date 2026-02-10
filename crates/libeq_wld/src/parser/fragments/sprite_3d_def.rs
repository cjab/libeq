use std::any::Any;

use super::common::{RenderInfo, RenderMethod};
use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// This fragment is poorly understood. It seems to contain 26 parameters, some of which
/// are DWORDS (32-bit integers) and some of which are FLOATS (32-bit floating-point values).
/// Until more is known, they are here described as Params[0..25] and their known values
/// are documented.
///
/// In main zone files, the name of this fragment always seems to be CAMERA_DUMMY.
///
/// **Type ID:** 0x08
pub struct Sprite3DDef {
    pub name_reference: StringReference,

    pub flags: ThreeDSpriteFlags,

    /// NUMVERTICES
    pub vertex_count: u32,

    /// NUMBSPNODES
    pub bsp_node_count: u32,

    /// SPHERELIST
    pub sphere_list_reference: u32,

    /// CENTEROFFSET %f %f %f
    pub center_offset: Option<(f32, f32, f32)>,

    /// BOUNDINGRADIUS %f
    pub bounding_radius: Option<f32>,

    /// XYZ %f %f %f
    pub vertices: Vec<(f32, f32, f32)>,

    // BSPNODE
    pub bsp_nodes: Vec<BspNodeEntry>,
}

impl FragmentParser for Sprite3DDef {
    type T = Self;

    const TYPE_ID: u32 = 0x08;
    const TYPE_NAME: &'static str = "Sprite3DDef";

    fn parse(input: &[u8]) -> WResult<'_, Sprite3DDef> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = ThreeDSpriteFlags::parse(i)?;
        let (i, vertex_count) = le_u32(i)?;
        let (i, bsp_node_count) = le_u32(i)?;
        let (i, sphere_list_reference) = le_u32(i)?;
        let (i, center_offset) = if flags.has_center_offset() {
            tuple((le_f32, le_f32, le_f32))(i).map(|(i, b)| (i, Some(b)))?
        } else {
            (i, None)
        };
        let (i, bounding_radius) = if flags.has_bounding_radius() {
            le_f32(i).map(|(i, b)| (i, Some(b)))?
        } else {
            (i, None)
        };
        let (i, vertices) = count(tuple((le_f32, le_f32, le_f32)), vertex_count as usize)(i)?;
        let (i, bsp_nodes) = count(BspNodeEntry::parse, bsp_node_count as usize)(i)?;

        Ok((
            i,
            Sprite3DDef {
                name_reference,
                flags,
                vertex_count,
                bsp_node_count,
                sphere_list_reference,
                center_offset,
                bounding_radius,
                vertices,
                bsp_nodes,
            },
        ))
    }
}

impl Fragment for Sprite3DDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.vertex_count.to_le_bytes()[..],
            &self.bsp_node_count.to_le_bytes()[..],
            &self.sphere_list_reference.to_le_bytes()[..],
            &self.center_offset.map_or(vec![], |c| {
                [c.0.to_le_bytes(), c.1.to_le_bytes(), c.2.to_le_bytes()].concat()
            })[..],
            &self
                .bounding_radius
                .map_or(vec![], |b| b.to_le_bytes().to_vec())[..],
            &self
                .vertices
                .iter()
                .flat_map(|(x, y, z)| [x.to_le_bytes(), y.to_le_bytes(), z.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .bsp_nodes
                .iter()
                .flat_map(|node| node.into_bytes())
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
pub struct BspNodeEntry {
    /// The number of vertex indices in this entry
    pub vertex_count: u32,

    pub front_tree: u32,

    pub back_tree: u32,

    pub vertex_indices: Vec<u32>,

    pub render_method: RenderMethod,

    pub render_info: RenderInfo,
}

impl BspNodeEntry {
    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, vertex_count) = le_u32(input)?;
        let (i, front_tree) = le_u32(i)?;
        let (i, back_tree) = le_u32(i)?;
        let (i, vertex_indices) = count(le_u32, vertex_count as usize)(i)?;
        let (i, render_method) = RenderMethod::parse(i)?;
        let (i, render_info) = RenderInfo::parse(i)?;

        Ok((
            i,
            Self {
                vertex_count,
                front_tree,
                back_tree,
                vertex_indices,
                render_method,
                render_info,
            },
        ))
    }

    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.vertex_count.to_le_bytes()[..],
            &self.front_tree.to_le_bytes()[..],
            &self.back_tree.to_le_bytes()[..],
            &self
                .vertex_indices
                .iter()
                .flat_map(|idx| idx.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &self.render_method.into_bytes()[..],
            &self.render_info.into_bytes()[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct ThreeDSpriteFlags(u32);

impl ThreeDSpriteFlags {
    const HAS_CENTER_OFFSET: u32 = 0x01;
    const HAS_BOUNDING_RADIUS: u32 = 0x02;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
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
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1729-0x08.frag")[..];
        let (remaining, frag) = Sprite3DDef::parse(data).unwrap();

        assert_eq!(remaining.len(), 0);

        assert_eq!(frag.name_reference, StringReference::new(-29305));
        assert_eq!(frag.flags, ThreeDSpriteFlags(0));
        assert_eq!(frag.vertex_count, 4);
        assert_eq!(frag.bsp_node_count, 1);
        assert_eq!(frag.sphere_list_reference, 0);
        assert_eq!(frag.center_offset, None);
        assert_eq!(frag.bounding_radius, None);
        assert_eq!(
            frag.vertices,
            vec![
                (0.0, -1.0, 1.0),
                (0.0, 1.0, 1.0),
                (0.0, 1.0, -1.0),
                (0.0, -1.0, -1.0)
            ]
        );
        assert_eq!(frag.bsp_nodes.len(), 1);
        assert_eq!(frag.bsp_nodes[0].vertex_count, 4);
        assert_eq!(frag.bsp_nodes[0].render_method, RenderMethod::from_u32(0));
        assert_eq!(frag.bsp_nodes[0].render_info.flags.has_pen(), true);
        assert_eq!(frag.bsp_nodes[0].render_info.flags.has_brightness(), false);
        assert_eq!(
            frag.bsp_nodes[0].render_info.flags.has_scaled_ambient(),
            false
        );
        assert_eq!(
            frag.bsp_nodes[0].render_info.flags.has_simple_sprite(),
            false
        );
        assert_eq!(frag.bsp_nodes[0].render_info.flags.is_two_sided(), false);
        assert_eq!(frag.bsp_nodes[0].render_info.pen, Some(11));
        assert_eq!(frag.bsp_nodes[0].render_info.flags.has_uv_info(), false);
        assert_eq!(frag.bsp_nodes[0].front_tree, 0);
        assert_eq!(frag.bsp_nodes[0].back_tree, 0);
        assert_eq!(frag.bsp_nodes[0].vertex_indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1729-0x08.frag")[..];
        let frag = Sprite3DDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
