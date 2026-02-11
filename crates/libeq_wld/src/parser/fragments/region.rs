use std::any::Any;

use super::{
    DmSpriteDef2, Fragment, FragmentParser, FragmentRef, RenderInfo, RenderMethod, StringReference,
    WResult,
};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_u8, le_u16, le_u32};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A region within a map's BSP Tree.
///
/// **Type ID:** 0x22
pub struct Region {
    pub name_reference: StringReference,

    /// bit 0 - has SPHERE %f %f %f %f
    /// bit 1 - has REVERBVOLUME %f
    /// bit 2 - has REVERBOFFSET %d
    /// bit 3 - has REGIONFOG
    /// bit 4 - has ENABLEGOURAUD2
    /// bit 5 - has ENCODEDVISIBILITY
    /// ...
    /// bit 7 (windcatcher) - If set then `pvs` contains u8 entries (more common).
    /// bit 8 - ??? seems to mean mesh_reference exists?
    ///
    /// WINDCATCHER:
    /// Usually contains 0x181 for regions that contain polygons and 0x81
    /// for regions that are empty.
    pub flags: RegionFlags,

    /// AMBIENTLIGHT %s
    pub ambient_light: FragmentRef<i32>,

    /// NUMREGIONVERTEX %d
    pub num_region_vertex: u32,

    /// NUMPROXIMALREGIONS %d
    pub num_proximal_regions: u32,

    /// NUMRENDERVERTICES %d
    pub num_render_vertices: u32,

    /// NUMWALLS %d
    pub num_walls: u32,

    /// NUMOBSTACLES %d
    pub num_obstacles: u32,

    /// NUMCUTTINGOBSTACLES %d
    pub num_cutting_obstacles: u32,

    /// NUMVISNODE %d
    pub num_vis_node: u32,

    /// potential visible set
    /// NUMVISLIST %d
    pub num_vis_list: u32,

    /// XYZ %f %f %f
    pub region_vertices: Vec<(f32, f32, f32)>,

    /// PROXIMALREGION %d %f
    pub proximal_regions: Vec<(u32, f32)>,

    /// XYZ %f %f %f
    pub render_vertices: Vec<(f32, f32, f32)>,

    /// WALL
    pub walls: Vec<Wall>,

    /// OBSTACLE
    pub obstacles: Vec<Obstacle>,

    /// VISNODE
    pub vis_nodes: Vec<VisNode>,

    /// VISIBLELIST
    pub visible_lists: Vec<VisibleList>,

    /// SPHERE %f %f %f %f
    pub sphere: Option<(f32, f32, f32, f32)>,

    /// REVERBVOLUME %f
    pub reverb_volume: Option<f32>,

    /// REVERBOFFSET %d
    pub reverb_offset: Option<i32>,

    /// Length of USERDATA string
    user_data_size: u32,

    /// USERDATA %s
    user_data: Vec<u8>,

    /// This does not appear in WLDCOM.
    /// WINDCATCHER:
    /// If there are any polygons in this region then this reference points to a [DmSpriteDef2]
    /// that contains only those polygons. That [DmSpriteDef2] must contain all geometry information
    /// contained within the volume that this region represents and nothing that lies outside of
    /// that volume.
    pub mesh_reference: Option<FragmentRef<DmSpriteDef2>>,
}

impl FragmentParser for Region {
    type T = Self;

    const TYPE_ID: u32 = 0x22;
    const TYPE_NAME: &'static str = "Region";

    fn parse(input: &[u8]) -> WResult<'_, Region> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = RegionFlags::parse(i)?;
        let (i, ambient_light) = FragmentRef::parse(i)?;
        let (i, num_region_vertex) = le_u32(i)?;
        let (i, num_proximal_regions) = le_u32(i)?;
        let (i, num_render_vertices) = le_u32(i)?;
        let (i, num_walls) = le_u32(i)?;
        let (i, num_obstacles) = le_u32(i)?;
        let (i, num_cutting_obstacles) = le_u32(i)?;
        let (i, num_vis_node) = le_u32(i)?;
        let (i, num_vis_list) = le_u32(i)?;
        let (i, region_vertices) =
            count((le_f32, le_f32, le_f32), num_region_vertex as usize).parse(i)?;
        let (i, proximal_regions) =
            count((le_u32, le_f32), num_proximal_regions as usize).parse(i)?;

        // Not 100% on the num_walls == 0 check. It looks like num_render_vertices can contain the sum of rendered wall vertices.
        // TODO: Find a region with both walls and render vertices
        let render_vertices_count = if num_walls == 0 {
            num_render_vertices
        } else {
            0
        };

        let (i, render_vertices) =
            count((le_f32, le_f32, le_f32), render_vertices_count as usize).parse(i)?;
        let (i, walls) = count(Wall::parse, num_walls as usize).parse(i)?;
        let (i, obstacles) = count(Obstacle::parse, num_obstacles as usize).parse(i)?;
        let (i, vis_nodes) = count(VisNode::parse, num_vis_node as usize).parse(i)?;
        let (i, visible_lists) = if flags.has_byte_entries() {
            count(VisibleList::parse_with_bytes, num_vis_list as usize).parse(i)?
        } else {
            count(VisibleList::parse_with_words, num_vis_list as usize).parse(i)?
        };

        let (i, sphere) = if flags.has_sphere() {
            (le_f32, le_f32, le_f32, le_f32)
                .parse(i)
                .map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };

        let (i, reverb_volume) = if flags.has_reverb_volume() {
            le_f32(i).map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };

        let (i, reverb_offset) = if flags.has_reverb_offset() {
            le_i32(i).map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };

        let (i, user_data_size) = le_u32(i)?;
        let (i, user_data) = count(le_u8, user_data_size as usize).parse(i)?;

        let (i, mesh_reference) = if flags.has_mesh_reference() {
            FragmentRef::parse(i).map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            Region {
                name_reference,
                flags,
                ambient_light,
                num_region_vertex,
                num_proximal_regions,
                num_render_vertices,
                num_walls,
                num_obstacles,
                num_cutting_obstacles,
                num_vis_node,
                num_vis_list,
                region_vertices,
                proximal_regions,
                render_vertices,
                walls,
                obstacles,
                vis_nodes,
                visible_lists,
                sphere,
                reverb_volume,
                reverb_offset,
                user_data_size,
                user_data,
                mesh_reference,
            },
        ))
    }
}

impl Fragment for Region {
    fn to_bytes(&self) -> Vec<u8> {
        let bytes = [
            &self.name_reference.to_bytes()[..],
            &self.flags.to_bytes()[..],
            &self.ambient_light.to_bytes()[..],
            &self.num_region_vertex.to_le_bytes()[..],
            &self.num_proximal_regions.to_le_bytes()[..],
            &self.num_render_vertices.to_le_bytes()[..],
            &self.num_walls.to_le_bytes()[..],
            &self.num_obstacles.to_le_bytes()[..],
            &self.num_cutting_obstacles.to_le_bytes()[..],
            &self.num_vis_node.to_le_bytes()[..],
            &self.num_vis_list.to_le_bytes()[..],
            &self
                .region_vertices
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .proximal_regions
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .render_vertices
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .walls
                .iter()
                .flat_map(|w| w.to_bytes())
                .collect::<Vec<_>>()[..],
            &self
                .obstacles
                .iter()
                .flat_map(|o| o.to_bytes())
                .collect::<Vec<_>>()[..],
            &self
                .vis_nodes
                .iter()
                .flat_map(|v| v.to_bytes())
                .collect::<Vec<_>>()[..],
            &self
                .visible_lists
                .iter()
                .flat_map(|v| v.to_bytes())
                .collect::<Vec<_>>()[..],
            &self.sphere.map_or(vec![], |s| {
                [
                    s.0.to_le_bytes(),
                    s.1.to_le_bytes(),
                    s.2.to_le_bytes(),
                    s.3.to_le_bytes(),
                ]
                .concat()
            }),
            &self
                .reverb_volume
                .map_or(vec![], |r| r.to_le_bytes().to_vec())[..],
            &self
                .reverb_offset
                .map_or(vec![], |r| r.to_le_bytes().to_vec())[..],
            &self.user_data_size.to_le_bytes()[..],
            &self.user_data[..],
            &self
                .mesh_reference
                .as_ref()
                .map_or(vec![], |m| m.to_bytes())[..],
        ]
        .concat();

        let padding_size = (4 - bytes.len() % 4) % 4;
        let padding: Vec<u8> = vec![0; padding_size];

        [&bytes[..], &padding[..]].concat()
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
pub struct RegionFlags(u32);

impl RegionFlags {
    const HAS_SPHERE: u32 = 0x01;
    const HAS_REVERB_VOLUME: u32 = 0x02;
    const HAS_REVERB_OFFSET: u32 = 0x04;
    const REGION_FOG: u32 = 0x08;
    const ENABLE_GOURAUD2: u32 = 0x10;
    const ENCODED_VISIBILITY: u32 = 0x20;
    //TODO: Verify these
    const HAS_BYTE_ENTRIES: u32 = 0x80;
    const HAS_MESH_REFERENCE: u32 = 0x100;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, raw_flags) = le_u32(input)?;
        Ok((i, Self(raw_flags)))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_sphere(&self) -> bool {
        self.0 & Self::HAS_SPHERE == Self::HAS_SPHERE
    }

    pub fn has_reverb_volume(&self) -> bool {
        self.0 & Self::HAS_REVERB_VOLUME == Self::HAS_REVERB_VOLUME
    }

    pub fn has_reverb_offset(&self) -> bool {
        self.0 & Self::HAS_REVERB_OFFSET == Self::HAS_REVERB_OFFSET
    }

    pub fn region_fog(&self) -> bool {
        self.0 & Self::REGION_FOG == Self::REGION_FOG
    }

    pub fn enable_gouraud2(&self) -> bool {
        self.0 & Self::ENABLE_GOURAUD2 == Self::ENABLE_GOURAUD2
    }

    pub fn encoded_visibility(&self) -> bool {
        self.0 & Self::ENCODED_VISIBILITY == Self::ENCODED_VISIBILITY
    }

    //TODO: Verify this
    pub fn has_byte_entries(&self) -> bool {
        self.0 & Self::HAS_BYTE_ENTRIES == Self::HAS_BYTE_ENTRIES
    }

    //TODO: Verify this
    pub fn has_mesh_reference(&self) -> bool {
        self.0 & Self::HAS_MESH_REFERENCE == Self::HAS_MESH_REFERENCE
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct Wall {
    /// bit 0 - has FLOOR (is floor?)
    /// bit 1 - has RENDERMETHOD and NORMALABCD (is renderable?)
    flags: WallFlags,

    /// NUMVERTICES %d
    num_vertices: u32,

    /// RENDERMETHOD ...
    render_method: Option<RenderMethod>,

    /// RENDERINFO
    render_info: Option<RenderInfo>,

    /// NORMALABCD %f %f %f %f
    normal_abcd: Option<(f32, f32, f32, f32)>,

    /// VERTEXLIST %d ...%d
    /// Binary values are 0 based. "VERTEXLIST 1" becomes vertex_list[0]
    vertex_list: Vec<u32>,
}

impl Wall {
    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, flags) = WallFlags::parse(input)?;
        let (i, num_vertices) = le_u32(i)?;
        let (i, vertex_list) = count(le_u32, num_vertices as usize).parse(i)?;

        let (i, render_method) = if flags.has_method_and_normal() {
            RenderMethod::parse(i).map(|(rem, m)| (rem, Some(m)))?
        } else {
            (i, None)
        };

        let (i, render_info) = if flags.has_method_and_normal() {
            RenderInfo::parse(i).map(|(rem, i)| (rem, Some(i)))?
        } else {
            (i, None)
        };

        let (i, normal_abcd) = if flags.has_method_and_normal() {
            (le_f32, le_f32, le_f32, le_f32)
                .parse(i)
                .map(|(rem, n)| (rem, Some(n)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            Self {
                flags,
                num_vertices,
                vertex_list,
                render_method,
                render_info,
                normal_abcd,
            },
        ))
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.flags.to_bytes()[..],
            &self.num_vertices.to_le_bytes()[..],
            &self
                .vertex_list
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &self.render_method.as_ref().map_or(vec![], |m| m.to_bytes())[..],
            &self
                .render_info
                .as_ref()
                .map_or(vec![], |i| i.to_bytes().to_vec())[..],
            &self.normal_abcd.map_or(vec![], |m| {
                [
                    m.0.to_le_bytes(),
                    m.1.to_le_bytes(),
                    m.2.to_le_bytes(),
                    m.3.to_le_bytes(),
                ]
                .concat()
            })[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct WallFlags(u32);

impl WallFlags {
    const HAS_FLOOR: u32 = 0x01;
    const HAS_METHOD_AND_NORMAL: u32 = 0x02;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, raw_flags) = le_u32(input)?;
        Ok((i, Self(raw_flags)))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_floor(&self) -> bool {
        self.0 & Self::HAS_FLOOR == Self::HAS_FLOOR
    }

    pub fn has_method_and_normal(&self) -> bool {
        self.0 & Self::HAS_METHOD_AND_NORMAL == Self::HAS_METHOD_AND_NORMAL
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// _Unknown_
pub struct Obstacle {
    /// bit 0 - is a FLOOR
    /// bit 1 - is a GEOMETRYCUTTINGOBSTACLE
    /// bit 2 - has USERDATA %s
    flags: ObstacleFlags,

    /// NEXTREGION %d
    next_region: i32,

    /// XY_VERTEX 0 %d
    /// XYZ_VERTEX 0 %d
    /// XY_LINE 0 %d %d
    /// XY_EDGE 0 %d %d
    /// XYZ_EDGE 0 %d %d
    /// PLANE 0 %d
    /// EDGEPOLYGON 0
    /// EDGEWALL 0 %d
    obstacle_type: ObstacleType,

    // NUMVERTICES %d
    num_vertices: Option<u32>,

    /// VERTEXLIST %d ...%d
    vertex_list: Option<Vec<u32>>,

    /// NORMALABCD %f %f %f %f
    normal_abcd: Option<(f32, f32, f32, f32)>,

    /// EDGEWALL 0 %d
    /// Binary values are 0 based. "EDGEWALL 0 1" becomes edge_wall[0]
    edge_wall: Option<u32>,

    /// Length of USERDATA string
    user_data_size: Option<u32>,

    /// USERDATA %s
    user_data: Option<Vec<u8>>,
}

impl Obstacle {
    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, flags) = ObstacleFlags::parse(input)?;
        let (i, next_region) = le_i32(i)?;
        let (i, obstacle_type) = le_i32(i)?;
        let obstacle_type = FromPrimitive::from_i32(obstacle_type).unwrap();

        let (i, num_vertices) = if obstacle_type == ObstacleType::EdgePolygon
            || obstacle_type == ObstacleType::EdgePolygonNormalAbcd
        {
            le_u32(i).map(|(i, vertex_list_size)| (i, Some(vertex_list_size)))?
        } else {
            (i, None)
        };

        let (i, vertex_list) = if let Some(vertex_list_size) = num_vertices {
            count(le_u32, vertex_list_size as usize)
                .parse(i)
                .map(|(rem, v)| (rem, Some(v)))?
        } else {
            (i, None)
        };

        let (i, normal_abcd) = if obstacle_type == ObstacleType::EdgePolygonNormalAbcd {
            (le_f32, le_f32, le_f32, le_f32)
                .parse(i)
                .map(|(rem, n)| (rem, Some(n)))?
        } else {
            (i, None)
        };

        let (i, edge_wall) = if obstacle_type == ObstacleType::EdgeWall {
            le_u32(i).map(|(i, w)| (i, Some(w)))?
        } else {
            (i, None)
        };

        let (i, user_data_size) = if flags.has_user_data() {
            le_u32(i).map(|(rem, u)| (rem, Some(u)))?
        } else {
            (i, None)
        };

        let (i, user_data) = if let Some(data_size) = user_data_size {
            count(le_u8, data_size as usize)
                .parse(i)
                .map(|(rem, u)| (rem, Some(u)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            Self {
                flags,
                next_region,
                obstacle_type,
                num_vertices,
                vertex_list,
                normal_abcd,
                edge_wall,
                user_data_size,
                user_data,
            },
        ))
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.flags.to_bytes()[..],
            &self.next_region.to_le_bytes()[..],
            &self.obstacle_type.to_bytes()[..],
            &self
                .num_vertices
                .map_or(vec![], |n| n.to_le_bytes().to_vec())[..],
            &self
                .vertex_list
                .as_ref()
                .map_or(vec![], |o| o.iter().flat_map(|v| v.to_le_bytes()).collect())[..],
            &self.normal_abcd.map_or(vec![], |m| {
                [
                    m.0.to_le_bytes(),
                    m.1.to_le_bytes(),
                    m.2.to_le_bytes(),
                    m.3.to_le_bytes(),
                ]
                .concat()
            })[..],
            &self.edge_wall.map_or(vec![], |w| w.to_le_bytes().to_vec())[..],
            &self
                .user_data_size
                .map_or(vec![], |u| u.to_le_bytes().to_vec())[..],
            &self.user_data.as_ref().map_or(vec![], |u| u.clone())[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct ObstacleFlags(u32);

impl ObstacleFlags {
    const IS_FLOOR: u32 = 0x01;
    const IS_GEOMETRY_CUTTING: u32 = 0x02;
    const HAS_USER_DATA: u32 = 0x04;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, raw_flags) = le_u32(input)?;
        Ok((i, Self(raw_flags)))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn is_floor(&self) -> bool {
        self.0 & Self::IS_FLOOR == Self::IS_FLOOR
    }

    pub fn is_geometry_cutting(&self) -> bool {
        self.0 & Self::IS_GEOMETRY_CUTTING == Self::IS_GEOMETRY_CUTTING
    }

    pub fn has_user_data(&self) -> bool {
        self.0 & Self::HAS_USER_DATA == Self::HAS_USER_DATA
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
enum ObstacleType {
    XyVertex = 8,
    XyzVertex = 9,
    XyLine = 10,
    XyEdge = 11,
    XyzEdge = 12,
    Plane = 13,
    EdgePolygon = 14,
    EdgeWall = 18,
    EdgePolygonNormalAbcd = -15,
}

impl ObstacleType {
    fn to_bytes(&self) -> Vec<u8> {
        (*self as i32).to_le_bytes().to_vec()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct VisNode {
    /// NORMALABCD %f %f %f %f
    normal_abcd: (f32, f32, f32, f32),

    /// VISLISTINDEX %d
    vis_list_index: u32,

    /// FRONTTREE %d
    front_tree: u32,

    /// BACKTREE %d
    back_tree: u32,
}

impl VisNode {
    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, normal_abcd) = (le_f32, le_f32, le_f32, le_f32).parse(input)?;
        let (i, vis_list_index) = le_u32(i)?;
        let (i, front_tree) = le_u32(i)?;
        let (i, back_tree) = le_u32(i)?;

        Ok((
            i,
            Self {
                normal_abcd,
                vis_list_index,
                front_tree,
                back_tree,
            },
        ))
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.normal_abcd.0.to_le_bytes()[..],
            &self.normal_abcd.1.to_le_bytes()[..],
            &self.normal_abcd.2.to_le_bytes()[..],
            &self.normal_abcd.3.to_le_bytes()[..],
            &self.vis_list_index.to_le_bytes()[..],
            &self.front_tree.to_le_bytes()[..],
            &self.back_tree.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct VisibleList {
    /// RANGE %d
    range_count: u16,

    /// ...%d
    /// This is a complicated field. It contains run-length-encoded data that tells the
    /// client which regions are “nearby”. The purpose appears to be so that the client
    /// can determine which mobs in the zone have to have their Z coordinates checked,
    /// so that they will fall to the ground (or until they land on something). Since
    /// it’s expensive to do this, it makes sense to only do it for regions that are
    /// visible to the player instead of doing it for all mobs in the entire zone (repeatedly).
    ///
    /// I’ve only encountered data where the stream is a list of BYTEs instead of WORDs.
    /// The following discussion describes RLE encoding a BYTE stream.
    ///
    /// The idea here is to form a sorted list of all region IDs that are within a
    /// certain distance, and then write that list as an RLE-encoded stream to save space.
    /// The procedure is as follows:
    ///
    /// 1. Set an initial region ID value to zero.
    /// 2. If this region ID is not present in the (sorted) list, skip forward to the first
    ///    one that is in the list. Write something to the stream that tells it how many IDs
    ///    were skipped.
    /// 3. Form a block of consecutive IDs that are in the list and write something to the
    ///    stream that tells the client that there are this many IDs that are in the list.
    /// 4. If there are more region IDs in the list, go back to step 2.
    ///
    /// When writing to the stream, either one or three bytes are written:
    ///
    /// * 0x00..0x3E - skip forward by this many region IDs
    /// * 0x3F, WORD - skip forward by the amount given in the following 16-bit WORD
    /// * 0x40..0x7F - skip forward based on bits 3..5, then include the number of
    ///                IDs based on bits 0..2
    /// * 0x80..0xBF - include the number of IDs based on bits 3..5, then skip forward
    ///                based on bits 0..2
    /// * 0xC0..0xFE - subtracting 0xC0, this many region IDs are nearby
    /// * 0xFF, WORD - the number of region IDs given by the following WORD are nearby
    ///
    /// It should be noted that the values in the range 0x40..0xBF allow skipping and
    /// including of no more than seven IDs at a time. Also, they are not necessary to
    /// encode a region list: they merely allow better compression.
    ranges: Vec<RangeEntry>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
enum RangeEntry {
    Byte(u8),
    Word(u16),
}

impl RangeEntry {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Byte(b) => vec![*b],
            Self::Word(u) => u.to_le_bytes().to_vec(),
        }
    }
}

impl VisibleList {
    fn parse_with_bytes(input: &[u8]) -> WResult<'_, Self> {
        Self::parse(input, true)
    }

    fn parse_with_words(input: &[u8]) -> WResult<'_, Self> {
        Self::parse(input, false)
    }

    fn parse(input: &[u8], byte_entries: bool) -> WResult<'_, Self> {
        let (i, range_count) = le_u16(input)?;

        let (i, ranges) = if byte_entries {
            count(le_u8, range_count as usize)
                .parse(i)
                .map(|(rem, e)| (rem, e.into_iter().map(RangeEntry::Byte).collect::<Vec<_>>()))?
        } else {
            count(le_u16, range_count as usize)
                .parse(i)
                .map(|(rem, e)| (rem, e.into_iter().map(RangeEntry::Word).collect::<Vec<_>>()))?
        };

        Ok((
            i,
            Self {
                range_count,
                ranges,
            },
        ))
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.range_count.to_le_bytes()[..],
            &self
                .ranges
                .iter()
                .flat_map(|r| r.to_bytes())
                .collect::<Vec<_>>(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{MaterialType, RenderInfoFlags, UvInfo, UvMap};

    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1731-0x22.frag")[..];
        let (remaining, frag) = Region::parse(data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(-29318));
        assert_eq!(frag.flags, RegionFlags(0x81));
        assert_eq!(frag.ambient_light, FragmentRef::new(0));
        assert_eq!(frag.num_region_vertex, 0);
        assert_eq!(frag.num_proximal_regions, 0);
        assert_eq!(frag.num_render_vertices, 0);
        assert_eq!(frag.num_walls, 0);
        assert_eq!(frag.num_obstacles, 0);
        assert_eq!(frag.num_cutting_obstacles, 0);
        assert_eq!(frag.num_vis_node, 1);
        assert_eq!(frag.num_vis_list, 1);
        assert_eq!(frag.region_vertices.len(), 0);
        assert_eq!(frag.proximal_regions.len(), 0);
        assert_eq!(frag.render_vertices.len(), 0);
        assert_eq!(frag.walls.len(), 0);
        assert_eq!(frag.obstacles.len(), 0);
        assert_eq!(frag.vis_nodes[0].normal_abcd, (0.0, 0.0, 0.0, 0.0));
        assert_eq!(frag.vis_nodes[0].vis_list_index, 1);
        assert_eq!(frag.vis_nodes[0].front_tree, 0);
        assert_eq!(frag.vis_nodes[0].back_tree, 0);
        assert_eq!(frag.visible_lists.len(), 1);
        assert_eq!(frag.visible_lists[0].range_count, 14);
        assert_eq!(
            frag.visible_lists[0].ranges,
            vec![
                254u8, 242, 24, 202, 86, 81, 39, 218, 87, 63, 44, 10, 19, 216
            ]
            .iter()
            .map(|i| RangeEntry::Byte(*i))
            .collect::<Vec<_>>()
        );
        assert_eq!(
            frag.sphere,
            Some((-2935.2515, -2823.152, -19.758118, 238.07394))
        );
        assert_eq!(frag.reverb_volume, None);
        assert_eq!(frag.reverb_offset, None);
        assert_eq!(frag.user_data_size, 0);
        assert_eq!(frag.user_data, vec![]);
        assert_eq!(frag.mesh_reference, None);
        assert_eq!(remaining, vec![]);
    }

    #[test]
    fn it_parses_with_mesh_reference() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1738-0x22.frag")[..];
        let (remaining, frag) = Region::parse(data).unwrap();
        assert_eq!(frag.mesh_reference, Some(FragmentRef::new(132)));
        assert_eq!(remaining, vec![]);
    }

    #[test]
    fn it_parses_with_padding() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/3260-0x22.frag")[..];
        let (remaining, _frag) = Region::parse(data).unwrap();
        assert_eq!(remaining, vec![0, 0, 0]);
    }

    #[test]
    fn it_parses_with_walls_and_obstructions() {
        let data =
            &include_bytes!("../../../fixtures/fragments/tanarus-thecity/8000-0x22.frag")[..];
        let (remaining, frag) = Region::parse(data).unwrap();

        // Walls
        assert_eq!(frag.walls.len(), 7);
        assert_eq!(frag.walls[0].flags, WallFlags(0x2));
        assert_eq!(frag.walls[0].flags.has_floor(), false);
        assert_eq!(frag.walls[0].num_vertices, 4);
        assert_eq!(frag.walls[0].normal_abcd, Some((0.0, 1.0, 0.0, -31.999935)));
        assert_eq!(frag.walls[0].vertex_list, vec![9700, 9590, 9574, 9687]);

        assert_eq!(
            frag.walls[0].render_method,
            Some(RenderMethod::UserDefined {
                material_type: MaterialType::Diffuse
            })
        );

        // Render Info
        let wall0_render_info = frag.walls[0].render_info.as_ref().unwrap();
        assert_eq!(wall0_render_info.flags, RenderInfoFlags::new(63));
        assert_eq!(wall0_render_info.flags.has_pen(), true);
        assert_eq!(wall0_render_info.flags.has_brightness(), true);
        assert_eq!(wall0_render_info.flags.has_scaled_ambient(), true);
        assert_eq!(wall0_render_info.flags.has_simple_sprite(), true);
        assert_eq!(wall0_render_info.flags.has_uv_info(), true);
        assert_eq!(wall0_render_info.flags.is_two_sided(), false);
        assert_eq!(wall0_render_info.pen, Some(201));
        assert_eq!(wall0_render_info.brightness, Some(1.0));
        assert_eq!(wall0_render_info.scaled_ambient, Some(1.0));
        assert_eq!(wall0_render_info.simple_sprite_reference, Some(7994));
        assert_eq!(
            wall0_render_info.uv_info,
            Some(UvInfo {
                uv_origin: (944.000061, 32.000000, -0.000015),
                u_axis: (-16.000000, -0.000001, 0.000000),
                v_axis: (0.000000, -0.000001, 16.000015)
            })
        );
        assert_eq!(
            wall0_render_info.uv_map,
            Some(UvMap {
                entry_count: 5,
                entries: vec![(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0), (0.0, 0.0)]
            })
        );

        // Floor wall
        assert_eq!(frag.walls[6].flags, WallFlags(0x3));
        assert_eq!(frag.walls[6].flags.has_floor(), true);
        assert_eq!(frag.walls[6].flags.has_method_and_normal(), true);
        assert_eq!(
            frag.walls[6].render_method,
            Some(RenderMethod::UserDefined {
                material_type: MaterialType::CompleteUnknown2
            })
        );

        // Obstacles
        assert_eq!(frag.obstacles.len(), 10);
        assert_eq!(frag.obstacles[0].flags.is_floor(), false);
        assert_eq!(
            frag.obstacles[0].obstacle_type,
            ObstacleType::EdgePolygonNormalAbcd
        );
        assert_eq!(
            frag.obstacles[0].normal_abcd,
            Some((1.0, 0.0, 0.0, -944.000061))
        );
        assert_eq!(frag.obstacles[0].num_vertices, Some(6));
        assert_eq!(
            frag.obstacles[0].vertex_list,
            Some(vec![9570, 9574, 9590, 9591, 9592, 9621])
        );
        assert_eq!(frag.obstacles[0].next_region, 1068);
        assert_eq!(frag.obstacles[0].edge_wall, None);

        assert_eq!(frag.obstacles[3].flags.is_floor(), false);
        assert_eq!(frag.obstacles[3].edge_wall, Some(0));

        assert_eq!(frag.obstacles[9].flags.is_floor(), true);
        assert_eq!(frag.obstacles[9].obstacle_type, ObstacleType::EdgeWall);
        assert_eq!(frag.obstacles[9].normal_abcd, None);
        assert_eq!(frag.obstacles[9].num_vertices, None);
        assert_eq!(frag.obstacles[9].vertex_list, None);
        assert_eq!(frag.obstacles[9].next_region, 0);
        assert_eq!(frag.obstacles[9].edge_wall, Some(6));

        assert_eq!(frag.user_data_size, 0);

        assert_eq!(remaining, vec![0, 0]);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1731-0x22.frag")[..];
        let frag = Region::parse(data).unwrap().1;

        assert_eq!(&frag.to_bytes()[..], data);
    }

    #[test]
    fn it_serializes_with_padding() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/3260-0x22.frag")[..];
        let frag = Region::parse(data).unwrap().1;

        assert_eq!(&frag.to_bytes()[..], data);
    }

    #[test]
    fn it_serializes_with_walls_and_obstructions() {
        let data =
            &include_bytes!("../../../fixtures/fragments/tanarus-thecity/8000-0x22.frag")[..];
        let frag = Region::parse(data).unwrap().1;

        assert_eq!(&frag.to_bytes()[..], data);
    }
}
