use std::any::Any;

use super::{
    fragment_ref, Fragment, FragmentRef, FragmentType, MaterialListFragment, StringReference,
};

use nom::combinator::map;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_i8, le_u16, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// This is the fragment most often used for models. However, [AlternateMeshFragment] fragment
/// is also sometimes used.
///
/// **Type ID:** 0x36
pub struct MeshFragment {
    pub name_reference: StringReference,

    /// _Unknown_ - The meaning of the flags is unknown but the following values
    /// have been observed:
    ///
    /// * For zone meshes: 0x00018003
    /// * For placeable objects: 0x00014003
    pub flags: u32,

    /// A reference to a [MaterialListFragment] fragment. This tells the client which materials
    /// this mesh uses.
    ///
    /// For zone meshes the [MaterialListFragment] contains all the materials used in the
    /// entire zone.
    ///
    /// For placeable objects the [MaterialListFragment] contains all of the materials used in
    /// that object.
    pub material_list_ref: FragmentRef<MaterialListFragment>,

    /// A reference to a [MeshAnimatedVerticesReferenceFragment]. This is set for non-character
    /// animated meshes. For example swaying flags and trees.
    pub animation_ref: FragmentRef<i32>,

    /// _Unknown_ - Usually empty
    pub fragment3: FragmentRef<i32>,

    /// _Unknown_ - This usually seems to reference the first [TextureImagesFragment] fragment in the file.
    pub fragment4: FragmentRef<i32>,

    /// For zone meshes this typically contains the X coordinate of the center of the mesh.
    /// This allows vertex coordinates in the mesh to be relative to the center instead of
    /// having absolute coordinates. This is important for preserving precision when encoding
    /// vertex coordinate values.
    ///
    /// For placeable objects this seems to define where the vertices will lie relative to
    /// the object’s local origin. This seems to allow placeable objects to be created that
    /// lie at some distance from their position as given in a [ObjectLocationFragment]
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
    /// the number of polygons, but this is by no means necessary as polygons can
    /// share vertices. However, sharing vertices degrades the ability to use vertex
    /// normals to make a mesh look more rounded (with shading).
    pub position_count: u16,

    /// The number of texture coordinate pairs there are in the mesh. This should
    /// equal the number of vertices in the mesh. Presumably this could contain zero
    /// if none of the polygons have textures mapped to them (but why would anyone do that?)
    pub texture_coordinate_count: u16,

    /// The number of vertex normal entries in the mesh. This should equal the number
    /// of vertices in the mesh. Presumably this could contain zero if vertices should
    /// use polygon normals instead, but I haven’t tried it (vertex normals are preferable
    /// anyway).
    pub normal_count: u16,

    /// The number of vertex color entries in the mesh. This should equal the number
    /// of vertices in the mesh, or zero if there are no vertex color entries.
    /// Meshes do not require color entries to work. Color entries are used for
    /// illuminating polygons when there is a nearby light source.
    pub color_count: u16,

    /// The number of polygons in the mesh.
    pub polygon_count: u16,

    /// This seems to only be used when dealing with animated (mob) models.
    /// It contains the number of vertex piece entries. Vertices are grouped together by
    /// skeleton piece in this case and vertex piece entries tell the client how
    /// many vertices are in each piece. It’s possible that there could be more
    /// pieces in the skeleton than are in the meshes it references. Extra pieces have
    /// no polygons or vertices and I suspect they are there to define attachment points for
    /// objects (e.g. weapons or shields).
    pub vertex_piece_count: u16,

    /// The number of polygon texture entries. Polygons are grouped together by
    /// material and polygon material entries. This tells the client the number of
    /// polygons using a material.
    pub polygon_material_count: u16,

    /// The number of vertex material entries. Vertices are grouped together
    /// by material and vertex material entries tell the client how many vertices there
    /// are using a material.
    pub vertex_material_count: u16,

    /// _Unknown_ - The number of entries in `data9`. Seems to be used only for
    /// animated mob models.
    pub size9: u16,

    /// This allows vertex coordinates to be stored as integral values instead of
    /// floating-point values, without losing precision based on mesh size. Vertex
    /// values are multiplied by (1 shl `scale`) and stored in the vertex entries.
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

    /// A collection of [MeshFragmentPolygonEntry]s used in this mesh.
    pub polygons: Vec<MeshFragmentPolygonEntry>,

    /// The first element of the tuple is the number of vertices in a skeleton piece.
    ///
    /// The second element of the tuple is the index of the piece according to the
    /// [SkeletonTrackSet] fragment. The very first piece (index 0) is usually not referenced here
    /// as it is usually jsut a "stem" starting point for the skeleton. Only those pieces
    /// referenced here in the mesh should actually be rendered. Any other pieces in the skeleton
    /// contain no vertices or polygons And have other purposes.
    pub vertex_pieces: Vec<(u16, u16)>,

    /// The first element of the tuple is the number of polygons that use the same material. All
    /// polygon entries are sorted by material index so that polygons use the same material are
    /// grouped together.
    ///
    /// The second element of the tuple is the index of the material that the polygons use according
    /// to the [MaterialListFragment] that this fragment references.
    pub polygon_materials: Vec<(u16, u16)>,

    /// The first element of the tuple is the number of vertices that use the same
    /// material. Vertex materials, like polygons, are sorted by material index so
    /// that vertices that use the same material are together.
    ///
    /// The second element of the tuple is the index of the material that the
    /// vertices use, according to the [MaterialListFragment] fragment that this fragment
    /// references.
    pub vertex_materials: Vec<(u16, u16)>,

    /// _Unknown_ - A collection of [MeshFragmentData9Entry]s
    pub data9: Vec<MeshFragmentData9Entry>,
}

impl FragmentType for MeshFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x36;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshFragment> {
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
                polygon_count,
                vertex_piece_count,
                polygon_material_count,
                vertex_material_count,
                size9,
                scale,
            ),
        ) = tuple((
            StringReference::parse,
            le_u32,
            fragment_ref,
            fragment_ref,
            fragment_ref,
            fragment_ref,
            tuple((le_f32, le_f32, le_f32)),
            tuple((le_u32, le_u32, le_u32)),
            le_f32,
            tuple((le_f32, le_f32, le_f32)),
            tuple((le_f32, le_f32, le_f32)),
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
        ))(input)?;

        let (
            remaining,
            (
                positions,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                polygons,
                vertex_pieces,
                polygon_materials,
                vertex_materials,
                data9,
            ),
        ) = tuple((
            count(tuple((le_i16, le_i16, le_i16)), position_count as usize),
            count(tuple((le_i16, le_i16)), texture_coordinate_count as usize),
            count(tuple((le_i8, le_i8, le_i8)), normal_count as usize),
            count(le_u32, color_count as usize),
            count(MeshFragmentPolygonEntry::parse, polygon_count as usize),
            count(tuple((le_u16, le_u16)), vertex_piece_count as usize),
            count(tuple((le_u16, le_u16)), polygon_material_count as usize),
            count(tuple((le_u16, le_u16)), vertex_material_count as usize),
            count(MeshFragmentData9Entry::parse, size9 as usize),
        ))(i)?;

        Ok((
            remaining,
            MeshFragment {
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
                polygon_count,
                vertex_piece_count,
                polygon_material_count,
                vertex_material_count,
                size9,
                scale,
                positions,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                polygons,
                vertex_pieces,
                polygon_materials,
                vertex_materials,
                data9,
            },
        ))
    }
}

impl Fragment for MeshFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.material_list_ref.serialize()[..],
            &self.fragment3.serialize()[..],
            &self.fragment4.serialize()[..],
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
            &self.polygon_count.to_le_bytes()[..],
            &self.vertex_piece_count.to_le_bytes()[..],
            &self.polygon_material_count.to_le_bytes()[..],
            &self.vertex_material_count.to_le_bytes()[..],
            &self.size9.to_le_bytes()[..],
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
                .polygons
                .iter()
                .flat_map(|p| p.serialize())
                .collect::<Vec<_>>()[..],
            &self
                .vertex_pieces
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .polygon_materials
                .iter()
                .flat_map(|p| [p.0.to_le_bytes(), p.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .vertex_materials
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .data9
                .iter()
                .flat_map(|d| d.serialize())
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
}

#[derive(Debug)]
/// Represents a polygon within a [MeshFragment].
pub struct MeshFragmentPolygonEntry {
    /// Most flags are _Unknown_. This usually contains 0x0 for polygons but
    /// contains 0x0010 for polygons that the player can pass through (like water
    /// and tree leaves).
    flags: u16,

    /// An index for each of the polygon's vertex coordinates (idx1, idx2, idx3).
    pub vertex_indexes: (u16, u16, u16),
}

impl MeshFragmentPolygonEntry {
    fn parse(input: &[u8]) -> IResult<&[u8], MeshFragmentPolygonEntry> {
        let (remaining, (flags, vertex_indexes)) =
            tuple((le_u16, tuple((le_u16, le_u16, le_u16))))(input)?;
        Ok((
            remaining,
            MeshFragmentPolygonEntry {
                flags,
                vertex_indexes,
            },
        ))
    }

    fn serialize(&self) -> Vec<u8> {
        [
            &self.flags.to_le_bytes()[..],
            &self.vertex_indexes.0.to_le_bytes()[..],
            &self.vertex_indexes.1.to_le_bytes()[..],
            &self.vertex_indexes.2.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct MeshFragmentData9Entry {
    /// _Unknown_ - This seems to reference one of the vertex entries. This field
    /// only exists if `type_field` contains a value in the range 1-3.
    pub index1: Option<u16>,

    /// _Unknown_ - This seems to reference one of the vertex entries. This field is only valid if
    /// `type_field` contains 1. Otherwise, this field must contain 0.
    pub index2: Option<u16>,

    /// _Unknown_ - If `type_field` contains 4, then this field exists instead of `index1`
    /// and `index2`. [MeshFragmentData9Entry]s seem to be sorted by this value.
    pub offset: Option<f32>,

    /// _Unknown_ - It seems to only contain values in the range 0-2.
    pub param1: u16,

    /// _Unknown_ - It seems to control whether `index1`, `index2`, and `offset` exist. It can only
    /// contain values in the range 1-4. It looks like the [MeshFragmentData9Entry]s are broken up into
    /// blocks, where each block is terminated by an entry where `type_field` is 4.
    pub type_field: u16,
}

impl MeshFragmentData9Entry {
    fn parse(input: &[u8]) -> IResult<&[u8], MeshFragmentData9Entry> {
        let (remaining, (index1, index2, offset, param1, type_field)) = tuple((
            map(le_u16, Some),
            map(le_u16, Some),
            map(le_f32, Some),
            le_u16,
            le_u16,
        ))(input)?;
        Ok((
            remaining,
            MeshFragmentData9Entry {
                index1,
                index2,
                offset,
                param1,
                type_field,
            },
        ))
    }

    fn serialize(&self) -> Vec<u8> {
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
