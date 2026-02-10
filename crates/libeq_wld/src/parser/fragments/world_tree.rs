use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, Region, StringReference, WResult};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A map's BSP Tree.
///
/// **Type ID:** 0x21
pub struct WorldTree {
    pub name_reference: StringReference,

    /// The number of [WorldNode]s in this tree.
    pub world_node_count: u32,

    /// The [WorldNode]s
    pub world_nodes: Vec<WorldNode>,
}

impl FragmentParser for WorldTree {
    type T = Self;

    const TYPE_ID: u32 = 0x21;
    const TYPE_NAME: &'static str = "WorldTree";

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, world_node_count) = le_u32(i)?;
        let (i, world_nodes) = count(WorldNode::parse, world_node_count as usize)(i)?;

        Ok((
            i,
            Self {
                name_reference,
                world_node_count,
                world_nodes,
            },
        ))
    }
}

impl Fragment for WorldTree {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.world_node_count.to_le_bytes()[..],
            &self
                .world_nodes
                .iter()
                .flat_map(|e| e.into_bytes())
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
/// Entries in the map's [WorldTree]
pub struct WorldNode {
    /// The normal to the split plane.
    pub normal: (f32, f32, f32),

    /// Distance from the split plane to the origin (0, 0, 0) in (x, y, z) space. With the above
    /// fields the splitting plane is represented in Hessian Normal Form.
    pub split_distance: f32,

    /// If this is a leaf node, this contains the index of the [Region] fragment that this
    /// refers to (with the lowest index being 1). Otherwise this will contain 0.
    pub region: FragmentRef<Region>,

    /// If this is not a leaf node these are references to [WorldNode] on either side of the
    /// splitting plane.
    pub front_tree: FragmentRef<WorldNode>,
    pub back_tree: FragmentRef<WorldNode>,
}

impl WorldNode {
    fn parse(input: &[u8]) -> WResult<'_, WorldNode> {
        let (i, normal) = tuple((le_f32, le_f32, le_f32))(input)?;
        let (i, split_distance) = le_f32(i)?;
        let (i, region) = FragmentRef::parse(i)?;
        let (i, front_tree) = FragmentRef::parse(i)?;
        let (i, back_tree) = FragmentRef::parse(i)?;

        Ok((
            i,
            Self {
                normal,
                split_distance,
                region,
                front_tree,
                back_tree,
            },
        ))
    }

    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.normal.0.to_le_bytes()[..],
            &self.normal.1.to_le_bytes()[..],
            &self.normal.2.to_le_bytes()[..],
            &self.split_distance.to_le_bytes()[..],
            &self.region.into_bytes()[..],
            &self.front_tree.into_bytes()[..],
            &self.back_tree.into_bytes()[..],
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1730-0x21.frag")[..];
        let frag = WorldTree::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0x0));
        assert_eq!(frag.world_node_count, 5809);
        assert_eq!(frag.world_nodes.len(), 5809);
        assert_eq!(frag.world_nodes[0].normal, (-1.0f32, 0.0f32, 0.0f32));
        assert_eq!(frag.world_nodes[0].split_distance, -187.8942f32);
        assert_eq!(frag.world_nodes[0].region, FragmentRef::new(0));
        assert_eq!(frag.world_nodes[0].front_tree, FragmentRef::new(2));
        assert_eq!(frag.world_nodes[0].back_tree, FragmentRef::new(2507));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1730-0x21.frag")[..];
        let frag = WorldTree::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
