use std::any::Any;

use super::{
    DmTrack, Fragment, FragmentParser, FragmentRef, MaterialPalette, StringReference, WResult,
};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i8, le_i16, le_u8, le_u16, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// This is the fragment most often used for models. However, [DmSpriteDef] fragment
/// is also sometimes used.
///
/// **Type ID:** 0x36
pub struct DmSpriteDef2 {
    pub name_reference: StringReference,

    /// _Unknown_ - The meaning of the flags is unknown but the following values
    /// have been observed:
    ///
    /// * For zone meshes: 0x00018003
    /// * For placeable objects: 0x00014003
    pub flags: u32,

    /// A reference to a [MaterialPalette] fragment. This tells the client which materials
    /// this mesh uses.
    ///
    /// For zone meshes the [MaterialPalette] contains all the materials used in the
    /// entire zone.
    ///
    /// For placeable objects the [MaterialPalette] contains all of the materials used in
    /// that object.
    pub material_list_ref: FragmentRef<MaterialPalette>,

    /// A reference to a [DmTrack]. This is set for non-character
    /// animated meshes. For example swaying flags and trees.
    pub animation_ref: FragmentRef<DmTrack>,

    /// _Unknown_ - Usually empty
    pub fragment3: FragmentRef<i32>,

    /// _Unknown_ - This usually seems to reference the first [BmInfo] fragment in the file.
    pub fragment4: FragmentRef<i32>,

    /// For zone meshes this typically contains the X coordinate of the center of the mesh.
    /// This allows vertex coordinates in the mesh to be relative to the center instead of
    /// having absolute coordinates. This is important for preserving precision when encoding
    /// vertex coordinate values.
    ///
    /// For placeable objects this seems to define where the vertices will lie relative to
    /// the object’s local origin. This seems to allow placeable objects to be created that
    /// lie at some distance from their position as given in a [Actor]
    /// (why one would do this is a mystery, though).
    pub center: (f32, f32, f32),

    /// _Unknown_ - Usually (0, 0, 0).
    pub params2: (u32, u32, u32),

    /// Given the values in `center`, this seems to contain the maximum distance between any
    /// vertex and that position. It seems to define a radius from that position within which
    /// the mesh lies.
    pub max_distance: f32,

    /// Contains min x, y, and z coords in absolute coords of any vertex in the mesh.
    pub min: (f32, f32, f32),

    /// Contains max x, y, and z coords in absolute coords of any vertex in the mesh.
    pub max: (f32, f32, f32),

    /// Tells how many vertices there are in the mesh. Normally this is three times
    /// the number of faces, but this is by no means necessary as faces can
    /// share vertices. However, sharing vertices degrades the ability to use vertex
    /// normals to make a mesh look more rounded (with shading).
    pub position_count: u16,

    /// The number of texture coordinate pairs there are in the mesh. This should
    /// equal the number of vertices in the mesh. Presumably this could contain zero
    /// if none of the faces have textures mapped to them (but why would anyone do that?)
    pub texture_coordinate_count: u16,

    /// The number of vertex normal entries in the mesh. This should equal the number
    /// of vertices in the mesh. Presumably this could contain zero if vertices should
    /// use polygon normals instead, but I haven’t tried it (vertex normals are preferable
    /// anyway).
    pub normal_count: u16,

    /// The number of vertex color entries in the mesh. This should equal the number
    /// of vertices in the mesh, or zero if there are no vertex color entries.
    /// Meshes do not require color entries to work. Color entries are used for
    /// illuminating faces when there is a nearby light source.
    pub color_count: u16,

    /// The number of faces in the mesh.
    pub face_count: u16,

    /// This seems to only be used when dealing with animated (mob) models.
    /// It contains the number of vertex piece entries. Vertices are grouped together by
    /// skeleton piece in this case and vertex piece entries tell the client how
    /// many vertices are in each piece. It’s possible that there could be more
    /// pieces in the skeleton than are in the meshes it references. Extra pieces have
    /// no faces or vertices and I suspect they are there to define attachment points for
    /// objects (e.g. weapons or shields).
    pub skin_assignment_groups_count: u16,

    /// The number of polygon texture entries. faces are grouped together by
    /// material and polygon material entries. This tells the client the number of
    /// faces using a material.
    pub face_material_groups_count: u16,

    /// The number of vertex material entries. Vertices are grouped together
    /// by material and vertex material entries tell the client how many vertices there
    /// are using a material.
    pub vertex_material_groups_count: u16,

    /// _Unknown_ - The number of entries in `meshops`. Seems to be used only for
    /// animated mob models.
    pub meshop_count: u16,

    /// This allows vertex coordinates to be stored as integral values instead of
    /// floating-point values, without losing precision based on mesh size. Vertex
    /// values are multiplied by (1 shl `scale`) and stored in the vertex entries.
    /// FPSCALE is the internal name.
    pub scale: u16,

    /// Vertices (x, y, z) belonging to this mesh. Each axis should
    /// be multiplied by (1 shl `scale`) for the final vertex position.
    pub positions: Vec<(i16, i16, i16)>,

    /// Texture coordinates (x, y) used to map textures to this mesh.
    ///
    /// Two formats are possible:
    /// * Old - Signed 16-bit texture value in pixels (most textures are 256 pixels in size).
    /// * New - Signed 32-bit value
    pub texture_coordinates: Vec<(i16, i16)>,

    /// Vertex normals (x, y, z). Each element contains a signed byte representing the
    /// component of the vertex normal, scaled such that –127 represents –1 and
    /// 127 represents 1.
    pub vertex_normals: Vec<(i8, i8, i8)>,

    /// This contains an RGBA color value for each vertex in the mesh.
    /// It specifies the additional color to be applied to the vertex, as
    /// if that vertex has been illuminated by a nearby light source. The A value
    /// isn’t fully understood; I believe it represents an alpha as applied to
    /// the texture, such that 0 makes the polygon a pure color and 0xFF either
    /// illuminates an unaltered texture or mutes the illumination completely.
    /// That is, it’s either a blending value or an alpha value. Further
    /// experimentation is required. 0xD9 seems to be a good (typical) A value for
    /// most illuminated vertices.
    pub vertex_colors: Vec<u32>,

    /// A collection of [DmSpriteDef2FaceEntry]s used in this mesh.
    pub faces: Vec<DmSpriteDef2FaceEntry>,

    /// The first element of the tuple is the number of vertices in a skeleton piece.
    ///
    /// The second element of the tuple is the index of the piece according to the
    /// [HierarchicalSpriteDef] fragment. The very first piece (index 0) is usually not referenced here
    /// as it is usually jsut a "stem" starting point for the skeleton. Only those pieces
    /// referenced here in the mesh should actually be rendered. Any other pieces in the skeleton
    /// contain no vertices or faces And have other purposes.
    pub skin_assignment_groups: Vec<(u16, u16)>,

    /// The first element of the tuple is the number of faces that use the same material. All
    /// polygon entries are sorted by material index so that faces use the same material are
    /// grouped together.
    ///
    /// The second element of the tuple is the index of the material that the faces use according
    /// to the [MaterialPalette] that this fragment references.
    pub face_material_groups: Vec<(u16, u16)>,

    /// The first element of the tuple is the number of vertices that use the same
    /// material. Vertex materials, like faces, are sorted by material index so
    /// that vertices that use the same material are together.
    ///
    /// The second element of the tuple is the index of the material that the
    /// vertices use, according to the [MaterialPalette] fragment that this fragment
    /// references.
    pub vertex_material_groups: Vec<(u16, u16)>,

    /// _Unknown_ - A collection of [DmSpriteDef2MeshOpEntry]s
    pub meshops: Vec<DmSpriteDef2MeshOpEntry>,
}

impl FragmentParser for DmSpriteDef2 {
    type T = Self;

    const TYPE_ID: u32 = 0x36;
    const TYPE_NAME: &'static str = "DmSpriteDef2";

    fn parse(input: &[u8]) -> WResult<'_, DmSpriteDef2> {
        let (
            i,
            (
                name_reference,
                flags,
                material_list_ref,
                animation_ref,
                fragment3,
                fragment4,
                center,
                params2,
                max_distance,
                min,
                max,
                position_count,
                texture_coordinate_count,
                normal_count,
                color_count,
                face_count,
                skin_assignment_groups_count,
                face_material_groups_count,
                vertex_material_groups_count,
                meshop_count,
                scale,
            ),
        ) = (
            StringReference::parse,
            le_u32,
            FragmentRef::parse,
            FragmentRef::parse,
            FragmentRef::parse,
            FragmentRef::parse,
            (le_f32, le_f32, le_f32),
            (le_u32, le_u32, le_u32),
            le_f32,
            (le_f32, le_f32, le_f32),
            (le_f32, le_f32, le_f32),
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
        )
            .parse(input)?;

        let (
            remaining,
            (
                positions,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                faces,
                skin_assignment_groups,
                face_material_groups,
                vertex_material_groups,
                meshops,
            ),
        ) = (
            count((le_i16, le_i16, le_i16), position_count as usize),
            count((le_i16, le_i16), texture_coordinate_count as usize),
            count((le_i8, le_i8, le_i8), normal_count as usize),
            count(le_u32, color_count as usize),
            count(DmSpriteDef2FaceEntry::parse, face_count as usize),
            count((le_u16, le_u16), skin_assignment_groups_count as usize),
            count((le_u16, le_u16), face_material_groups_count as usize),
            count((le_u16, le_u16), vertex_material_groups_count as usize),
            count(DmSpriteDef2MeshOpEntry::parse, meshop_count as usize),
        )
            .parse(i)?;

        // Note - There's trailing zeroes here which are not read.

        Ok((
            remaining,
            DmSpriteDef2 {
                name_reference,
                flags,
                material_list_ref,
                animation_ref,
                fragment3,
                fragment4,
                center,
                params2,
                max_distance,
                min,
                max,
                position_count,
                texture_coordinate_count,
                normal_count,
                color_count,
                face_count,
                skin_assignment_groups_count,
                face_material_groups_count,
                vertex_material_groups_count,
                meshop_count,
                scale,
                positions,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                faces,
                skin_assignment_groups,
                face_material_groups,
                vertex_material_groups,
                meshops,
            },
        ))
    }
}

impl Fragment for DmSpriteDef2 {
    fn to_bytes(&self) -> Vec<u8> {
        let meshops = &self
            .meshops
            .iter()
            .flat_map(|d| d.to_bytes())
            .collect::<Vec<_>>()[..];
        let padding_size = (4 - meshops.len() % 4) % 4;
        let padding: Vec<u8> = vec![0; padding_size];

        [
            &self.name_reference.to_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.material_list_ref.to_bytes()[..],
            &self.animation_ref.to_bytes()[..],
            &self.fragment3.to_bytes()[..],
            &self.fragment4.to_bytes()[..],
            &self.center.0.to_le_bytes()[..],
            &self.center.1.to_le_bytes()[..],
            &self.center.2.to_le_bytes()[..],
            &self.params2.0.to_le_bytes()[..],
            &self.params2.1.to_le_bytes()[..],
            &self.params2.2.to_le_bytes()[..],
            &self.max_distance.to_le_bytes()[..],
            &self.min.0.to_le_bytes()[..],
            &self.min.1.to_le_bytes()[..],
            &self.min.2.to_le_bytes()[..],
            &self.max.0.to_le_bytes()[..],
            &self.max.1.to_le_bytes()[..],
            &self.max.2.to_le_bytes()[..],
            &self.position_count.to_le_bytes()[..],
            &self.texture_coordinate_count.to_le_bytes()[..],
            &self.normal_count.to_le_bytes()[..],
            &self.color_count.to_le_bytes()[..],
            &self.face_count.to_le_bytes()[..],
            &self.skin_assignment_groups_count.to_le_bytes()[..],
            &self.face_material_groups_count.to_le_bytes()[..],
            &self.vertex_material_groups_count.to_le_bytes()[..],
            &self.meshop_count.to_le_bytes()[..],
            &self.scale.to_le_bytes()[..],
            &self
                .positions
                .iter()
                .flat_map(|p| [p.0.to_le_bytes(), p.1.to_le_bytes(), p.2.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .texture_coordinates
                .iter()
                .flat_map(|t| [t.0.to_le_bytes(), t.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .vertex_normals
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .vertex_colors
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &self
                .faces
                .iter()
                .flat_map(|p| p.to_bytes())
                .collect::<Vec<_>>()[..],
            &self
                .skin_assignment_groups
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .face_material_groups
                .iter()
                .flat_map(|p| [p.0.to_le_bytes(), p.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .vertex_material_groups
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            meshops,
            &padding[..],
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
/// Represents a polygon within a [DmSpriteDef2].
pub struct DmSpriteDef2FaceEntry {
    /// Most flags are _Unknown_. This usually contains 0x0 for faces but
    /// contains 0x0010 for faces that the player can pass through (like water
    /// and tree leaves).
    pub flags: u16,

    /// An index for each of the polygon's vertex coordinates (idx1, idx2, idx3).
    pub vertex_indexes: (u16, u16, u16),
}

impl DmSpriteDef2FaceEntry {
    fn parse(input: &[u8]) -> WResult<'_, DmSpriteDef2FaceEntry> {
        let (remaining, (flags, vertex_indexes)) =
            (le_u16, (le_u16, le_u16, le_u16)).parse(input)?;
        Ok((
            remaining,
            DmSpriteDef2FaceEntry {
                flags,
                vertex_indexes,
            },
        ))
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.flags.to_le_bytes()[..],
            &self.vertex_indexes.0.to_le_bytes()[..],
            &self.vertex_indexes.1.to_le_bytes()[..],
            &self.vertex_indexes.2.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// _Unknown_
pub struct DmSpriteDef2MeshOpEntry {
    /// _Unknown_ - This seems to reference one of the vertex entries. This field is only valid if
    /// `type_field` contains 1. Otherwise, this field must contain 0.
    pub index1: Option<u16>,

    /// _Unknown_ - This seems to reference one of the vertex entries. This field is only valid if
    /// `type_field` contains 1. Otherwise, this field must contain 0.
    pub index2: Option<u16>,

    /// _Unknown_ - If `type_field` contains 4, then this field exists instead of `index1`
    /// and `index2`. [DmSpriteDef2MeshOpEntry]s seem to be sorted by this value.
    pub offset: Option<f32>,

    /// _Unknown_ - It seems to only contain values in the range 0-2.
    pub param1: u8,

    /// _Unknown_ - It seems to control whether `index1`, `index2`, and `offset` exist. It can only
    /// contain values in the range 1-4. It looks like the [DmSpriteDef2MeshOpEntry]s are broken up into
    /// blocks, where each block is terminated by an entry where `type_field` is 4.
    ///
    /// The type of MESHOP, one of:
    /// 1: SW (vertex_index: u16, vertex_index: u16, type: u8) e.g. "MESHOP_SW 1553 1 1569" where the arguments are re-arranged to 1553 1569 0
    /// 2: FA (face_index: u16) + 3 empty bytes
    /// 3: VA (vertex_index: u16) + 3 empty bytes
    /// 4: EL (offset: f32) + 1 empty byte
    pub type_field: u8,
}

impl DmSpriteDef2MeshOpEntry {
    fn parse(input: &[u8]) -> WResult<'_, DmSpriteDef2MeshOpEntry> {
        let unknown_data = &input[0..4];
        let input = &input[4..];

        let (i, (param1, type_field)) = (le_u8, le_u8).parse(input)?;

        let (unknown_data, offset) = if type_field == 4 {
            le_f32(unknown_data).map(|(i, offset)| (i, Some(offset)))?
        } else {
            (unknown_data, None)
        };

        let (_, (index1, index2)) = if type_field != 4 {
            (le_u16, le_u16)
                .parse(unknown_data)
                .map(|(i, (index1, index2))| (i, (Some(index1), Some(index2))))?
        } else {
            (unknown_data, (None, None))
        };

        Ok((
            i,
            DmSpriteDef2MeshOpEntry {
                index1,
                index2,
                offset,
                param1,
                type_field,
            },
        ))
    }

    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.index1.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &self.index2.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &self.offset.map_or(vec![], |o| o.to_le_bytes().to_vec())[..],
            &self.param1.to_le_bytes()[..],
            &self.type_field.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0131-0x36.frag")[..];
        let frag = DmSpriteDef2::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-1134));
        assert_eq!(frag.flags, 0x18003);
        assert_eq!(frag.material_list_ref, FragmentRef::new(131));
        assert_eq!(frag.animation_ref, FragmentRef::new(0));
        assert_eq!(frag.fragment3, FragmentRef::new(0));
        assert_eq!(frag.fragment4, FragmentRef::new(-2));
        assert_eq!(frag.center, (-2502.0, -2432.0, 190.0));
        assert_eq!(frag.params2, (0, 0, 0));
        assert_eq!(frag.max_distance, 37.817947);
        assert_eq!(frag.min, (0.0, 0.0, 0.0));
        assert_eq!(frag.max, (0.0, 0.0, 0.0));
        assert_eq!(frag.position_count, 8);
        assert_eq!(frag.texture_coordinate_count, 8);
        assert_eq!(frag.normal_count, 8);
        assert_eq!(frag.face_count, 6);
        assert_eq!(frag.skin_assignment_groups_count, 0);
        assert_eq!(frag.face_material_groups_count, 1);
        assert_eq!(frag.vertex_material_groups_count, 1);
        assert_eq!(frag.meshop_count, 0);
        assert_eq!(frag.scale, 5);
        assert_eq!(frag.positions.len(), 8);
        assert_eq!(frag.positions[0], (2, -1154, -3));
        assert_eq!(frag.texture_coordinates.len(), 8);
        assert_eq!(frag.texture_coordinates[0], (77, 77));
        assert_eq!(frag.vertex_normals.len(), 8);
        assert_eq!(frag.vertex_normals[0], (29, 31, 119));
        assert_eq!(frag.vertex_colors.len(), 8);
        assert_eq!(frag.vertex_colors[0], 4043374848);
        assert_eq!(frag.faces.len(), 6);
        assert_eq!(frag.faces[0].flags, 0);
        assert_eq!(frag.faces[0].vertex_indexes, (0, 1, 2));
        assert_eq!(frag.skin_assignment_groups.len(), 0);
        assert_eq!(frag.face_material_groups.len(), 1);
        assert_eq!(frag.face_material_groups[0], (6, 0));
        assert_eq!(frag.vertex_material_groups.len(), 1);
        assert_eq!(frag.vertex_material_groups[0], (8, 0));
        assert_eq!(frag.meshops.len(), 0);
    }

    #[test]
    fn it_parses_meshops() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/global_chr/0177-0x36.frag")[..];
        let frag = DmSpriteDef2::parse(data).unwrap().1;

        assert_eq!(frag.meshop_count, 1387);
        assert_eq!(frag.meshops.len(), 1387);

        for item in frag.meshops.iter() {
            assert!(item.type_field <= 4);
        }

        assert_eq!(frag.meshops[0].type_field, 2);
        assert_eq!(frag.meshops[0].index1.unwrap(), 4);
        assert_eq!(frag.meshops[0].index2.unwrap(), 0);

        assert_eq!(frag.meshops[1].type_field, 3);

        assert_eq!(frag.meshops[5].type_field, 4);
        assert_eq!(frag.meshops[5].offset.unwrap(), 1.0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0131-0x36.frag")[..];
        let frag = DmSpriteDef2::parse(data).unwrap().1;

        assert_eq!(&frag.to_bytes()[..], data);
    }

    #[test]
    fn it_serializes_meshops() {
        let data = &include_bytes!("../../../fixtures/fragments/global_chr/0177-0x36.frag")[..];
        let frag = DmSpriteDef2::parse(data).unwrap().1;

        assert_eq!(&frag.to_bytes()[..], data);
    }
}
