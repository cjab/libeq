use std::marker::PhantomData;

use nom::combinator::map;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_i32, le_i8, le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

use super::decode_string;

#[derive(Debug, Clone, Copy)]
pub struct FragmentRef<T>(pub i32, PhantomData<T>);

impl<T> FragmentRef<T> {
    pub fn new(idx: i32) -> FragmentRef<T> {
        FragmentRef(idx, PhantomData)
    }

    pub fn is_name_ref(&self) -> bool {
        self.0 <= 0
    }

    pub fn is_index_ref(&self) -> bool {
        self.0 > 0
    }
}

pub trait Fragment {
    type T;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::T>;
}

fn fragment_ref<T>(input: &[u8]) -> IResult<&[u8], FragmentRef<T>> {
    let (remaining, frag_ref_idx) = le_i32(input)?;
    Ok((remaining, FragmentRef(frag_ref_idx, PhantomData)))
}

#[derive(Debug)]
/// A map's BSP Tree.
///
/// **Type ID:** 0x21
pub struct BspTreeFragment {
    /// The number of [BspTreeFragmentEntry]s in this tree.
    size1: u32,

    /// The [BspTreeFragmentEntry]s
    pub entries: Vec<BspTreeFragmentEntry>,
}

impl Fragment for BspTreeFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], BspTreeFragment> {
        let (i, size1) = le_u32(input)?;
        let (remaining, entries) = count(BspTreeFragmentEntry::parse, size1 as usize)(i)?;

        Ok((remaining, BspTreeFragment { size1, entries }))
    }
}

#[derive(Debug)]
/// Entries in the map's [BspTreeFragment]
pub struct BspTreeFragmentEntry {
    /// The normal to the split plane.
    pub normal: (f32, f32, f32),

    /// Distance from the split plane to the origin (0, 0, 0) in (x, y, z) space. With the above
    /// fields the splitting plane is represented in Hessian Normal Form.
    pub split_distance: f32,

    /// If this is a leaf node, this contains the index of the [BspRegionFragment] fragment that this
    /// refers to (with the lowest index being 1). Otherwise this will contain 0.
    pub region: FragmentRef<BspRegionFragment>,

    /// If this is not a leaf node these are references to [BspTreeFragmentEntry] on either size of the
    /// splitting plane.
    pub nodes: (
        FragmentRef<BspTreeFragmentEntry>,
        FragmentRef<BspTreeFragmentEntry>,
    ),
}

impl Fragment for BspTreeFragmentEntry {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], BspTreeFragmentEntry> {
        let (remaining, (normal, split_distance, region, nodes)) = tuple((
            tuple((le_f32, le_f32, le_f32)),
            le_f32,
            fragment_ref,
            tuple((fragment_ref, fragment_ref)),
        ))(input)?;

        Ok((
            remaining,
            BspTreeFragmentEntry {
                normal,
                split_distance,
                region,
                nodes,
            },
        ))
    }
}

#[derive(Debug)]
/// A region within a map's BSP Tree.
///
/// **Type ID:** 0x22
pub struct BspRegionFragment {
    /// Most flags are _unknown_. Usually contains 0x181 for regions that contain polygons and 0x81
    /// for regions that are empty.
    /// * bit 5 - If set then `data6` contains u32 entries.
    /// * bit 7 - If set then `data6` contains u8 entries (more common).
    flags: u32,

    /// _Unknown_ - Some sort of fragment reference. Usually nothing is referenced.
    fragment1: FragmentRef<i32>,

    /// The number of bytes in `data1`
    size1: u32,

    /// The number of bytes in `data2`
    size2: u32,

    /// _Unknown_ - Usually 0
    params1: u32,

    /// The number of `data3` entries. Usually 0.
    size3: u32,

    /// The number of `data4` entries. Usually 0.
    size4: u32,

    /// _Unknown_ - Usually 0.
    params2: u32,

    /// The number of `data5` entries. Usually 1.
    size5: u32,

    /// The number of `data6` entries. Usually 1.
    size6: u32,

    /// According to the ZoneConverter source there are 12 * `size1` bytes here. Their format is
    /// _unknown_ for lack of sample data to figure it out.
    data1: Vec<u8>,

    /// According to the ZoneConverter source there are 8 * `size2` bytes here. Their format is
    /// _unknown_ for lack of sample data to figure it out.
    data2: Vec<u8>,

    /// _Unknown_ data entries
    data3: Vec<BspRegionFragmentData3Entry>,

    /// _Unknown_ data entries
    data4: Vec<BspRegionFragmentData4Entry>,

    /// _Unknown_ data entries
    data5: Vec<BspRegionFragmentData5Entry>,

    /// _Unknown_ data entries
    data6: Vec<BspRegionFragmentData6Entry>,

    /// The number of bytes in the `name7` field.
    size7: u32,

    /// _Unknown_ - An encoded string.
    name7: Vec<u8>,

    /// _Unknown_ - Usually references nothing.
    fragment2: FragmentRef<i32>,

    /// If there are any polygons in this region then this reference points to a [MeshFragment]
    /// that contains only those polygons. That [MeshFragment] must contain all geometry information
    /// contained within the volume that this region represents and nothing that lies outside of
    /// that volume.
    pub fragment3: Option<FragmentRef<MeshFragment>>,
}

impl Fragment for BspRegionFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragment> {
        let (i, (flags, fragment1, size1, size2, params1, size3, size4, params2, size5, size6)) =
            tuple((
                le_u32,
                fragment_ref,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
            ))(input)?;
        let (i, (data1, data2, data3, data4, data5, data6, size7)) = tuple((
            count(le_u8, size1 as usize),
            count(le_u8, size2 as usize),
            count(BspRegionFragmentData3Entry::parse, size3 as usize),
            count(BspRegionFragmentData4Entry::parse, size4 as usize),
            count(BspRegionFragmentData5Entry::parse, size5 as usize),
            count(BspRegionFragmentData6Entry::parse, size6 as usize),
            le_u32,
        ))(i)?;
        let (i, (name7, fragment2)) = tuple((count(le_u8, 12), fragment_ref))(i)?;

        let (remaining, fragment3) = if (flags & 0x100) == 0x100 {
            fragment_ref(i).map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            BspRegionFragment {
                flags,
                fragment1,
                size1,
                size2,
                params1,
                size3,
                size4,
                params2,
                size5,
                size6,
                data1,
                data2,
                data3,
                data4,
                data5,
                data6,
                size7,
                name7,
                fragment2,
                fragment3,
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct BspRegionFragmentData3Entry {
    /// _Unknown_
    /// * bit 1 - If set then the `params1`and `params2` fields exist.
    flags: u32,

    /// The number of `data1` entries.
    size1: u32,

    /// _Unknown_
    data1: Vec<u32>,

    /// _Unknown_ - Only exists if bit 1 of `flags` is set.
    params1: Option<(u32, u32, u32)>,

    /// _Unknown_ - Only exists if bit 1 of `flags` is set.
    params2: Option<u32>,
}

impl Fragment for BspRegionFragmentData3Entry {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentData3Entry> {
        let (i, (flags, size1)) = tuple((le_u32, le_u32))(input)?;
        let (i, data1) = count(le_u32, size1 as usize)(i)?;

        let has_params = flags & 0x02 == 0x02;
        let (remaining, (params1, params2)) = if has_params {
            tuple((
                map(tuple((le_u32, le_u32, le_u32)), Some),
                map(le_u32, Some),
            ))(i)?
        } else {
            (i, (None, None))
        };

        Ok((
            remaining,
            BspRegionFragmentData3Entry {
                flags,
                size1,
                data1,
                params1,
                params2,
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct BspRegionFragmentData4Entry {
    /// _Unknown_
    flags: u32,

    /// _Unknown_
    params1: u32,

    /// _Unknown_ - This seems to determine if `params2a` and/or `params2b` exist.
    type_field: u32,

    /// _Unknown_ - Only exists if `type_field` is greater than 7.
    params2a: Option<u32>,

    /// _Unknown_ - Only exists if `type_field` is one of the following:
    /// * 0x0A
    /// * 0x0B
    /// * 0x0C
    /// Though I'm not at all sure about this due to lack of sample data.
    params2b: Option<u32>,

    /// The number of bytes in the `name` field.
    name_size: u32,

    /// An encoded string.
    name: String,
}

impl Fragment for BspRegionFragmentData4Entry {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentData4Entry> {
        let (i, (flags, params1, type_field)) = tuple((le_u32, le_u32, le_u32))(input)?;

        let (i, params2a) = if type_field > 7 {
            map(le_u32, Some)(i)?
        } else {
            (i, None)
        };

        let (i, params2b) = if type_field > 7 {
            map(le_u32, Some)(i)?
        } else {
            (i, None)
        };

        let (i, name_size) = le_u32(i)?;

        let (remaining, name) = count(le_u8, name_size as usize)(i)?;

        Ok((
            remaining,
            BspRegionFragmentData4Entry {
                flags,
                params1,
                type_field,
                params2a,
                params2b,
                name_size,
                name: String::from_utf8(name).unwrap(),
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct BspRegionFragmentData5Entry {
    /// _Unknown_ - Usually 0.
    params1: (u32, u32, u32),

    /// _Unknown_ - Usually 0.
    params2: u32,

    /// _Unknown_ - Usually 1.
    params3: u32,

    /// _Unknown_ - Usually 0.
    params4: u32,

    /// _Unknown_ - Usually 0.
    params5: u32,
}

impl Fragment for BspRegionFragmentData5Entry {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentData5Entry> {
        let (remaining, (params1, params2, params3, params4, params5)) = tuple((
            tuple((le_u32, le_u32, le_u32)),
            le_u32,
            le_u32,
            le_u32,
            le_u32,
        ))(input)?;

        Ok((
            remaining,
            BspRegionFragmentData5Entry {
                params1,
                params2,
                params3,
                params4,
                params5,
            },
        ))
    }
}

#[derive(Debug)]
/// _Unknown_
pub struct BspRegionFragmentData6Entry {
    /// The number of entries in the `data` field
    size1: u16,

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
    data: Vec<u8>,
}

impl Fragment for BspRegionFragmentData6Entry {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentData6Entry> {
        let (i, size1) = le_u16(input)?;
        let (remaining, data) = count(le_u8, size1 as usize)(i)?;

        Ok((remaining, BspRegionFragmentData6Entry { size1, data }))
    }
}

#[derive(Debug)]
/// This is the fragment most often used for models. However, [AlternateMeshFragment] fragment
/// is also sometimes used.
///
/// **Type ID:** 0x36
pub struct MeshFragment {
    /// _Unknown_ - The meaning of the flags is unknown but the following values
    /// have been observed:
    ///
    /// * For zone meshes: 0x00018003
    /// * For placeable objects: 0x00014003
    flags: u32,

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
    fragment2: FragmentRef<i32>,

    /// _Unknown_ - Usually empty
    fragment3: FragmentRef<i32>,

    /// _Unknown_ - This usually seems to reference the first [TextureImagesFragment] fragment in the file.
    fragment4: FragmentRef<i32>,

    /// For zone meshes this typically contains the X coordinate of the center of the mesh.
    /// This allows vertex coordinates in the mesh to be relative to the center instead of
    /// having absolute coordinates. This is important for preserving precision when encoding
    /// vertex coordinate values.
    ///
    /// For placeable objects this seems to define where the vertices will lie relative to
    /// the object’s local origin. This seems to allow placeable objects to be created that
    /// lie at some distance from their position as given in a [ObjectLocation] fragment
    /// (why one would do this is a mystery, though).
    pub center: (f32, f32, f32),

    /// _Unknown_ - Usually (0, 0, 0).
    params2: (u32, u32, u32),

    /// Given the values in `center`, this seems to contain the maximum distance between any
    /// vertex and that position. It seems to define a radius from that position within which
    /// the mesh lies.
    max_distance: f32,

    /// Contains min x, y, and z coords in absolute coords of any vertex in the mesh.
    pub min: (f32, f32, f32),

    /// Contains max x, y, and z coords in absolute coords of any vertex in the mesh.
    pub max: (f32, f32, f32),

    /// Tells how many vertices there are in the mesh. Normally this is three times
    /// the number of polygons, but this is by no means necessary as polygons can
    /// share vertices. However, sharing vertices degrades the ability to use vertex
    /// normals to make a mesh look more rounded (with shading).
    position_count: u16,

    /// The number of texture coordinate pairs there are in the mesh. This should
    /// equal the number of vertices in the mesh. Presumably this could contain zero
    /// if none of the polygons have textures mapped to them (but why would anyone do that?)
    texture_coordinate_count: u16,

    /// The number of vertex normal entries in the mesh. This should equal the number
    /// of vertices in the mesh. Presumably this could contain zero if vertices should
    /// use polygon normals instead, but I haven’t tried it (vertex normals are preferable
    /// anyway).
    normal_count: u16,

    /// The number of vertex color entries in the mesh. This should equal the number
    /// of vertices in the mesh, or zero if there are no vertex color entries.
    /// Meshes do not require color entries to work. Color entries are used for
    /// illuminating polygons when there is a nearby light source.
    color_count: u16,

    /// The number of polygons in the mesh.
    polygon_count: u16,

    /// This seems to only be used when dealing with animated (mob) models.
    /// It contains the number of vertex piece entries. Vertices are grouped together by
    /// skeleton piece in this case and vertex piece entries tell the client how
    /// many vertices are in each piece. It’s possible that there could be more
    /// pieces in the skeleton than are in the meshes it references. Extra pieces have
    /// no polygons or vertices and I suspect they are there to define attachment points for
    /// objects (e.g. weapons or shields).
    vertex_piece_count: u16,

    /// The number of polygon texture entries. Polygons are grouped together by
    /// material and polygon material entries. This tells the client the number of
    /// polygons using a material.
    polygon_material_count: u16,

    /// The number of vertex material entries. Vertices are grouped together
    /// by material and vertex material entries tell the client how many vertices there
    /// are using a material.
    vertex_material_count: u16,

    /// _Unknown_ - The number of entries in `data9`. Seems to be used only for
    /// animated mob models.
    size9: u16,

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
    vertex_colors: Vec<u32>,

    /// A collection of [MeshFragmentPolygonEntry]s used in this mesh.
    pub polygons: Vec<MeshFragmentPolygonEntry>,

    /// The first element of the tuple is the number of vertices in a skeleton piece.
    ///
    /// The second element of the tuple is the index of the piece according to the
    /// [SkeletonTrackSet] fragment. The very first piece (index 0) is usually not referenced here
    /// as it is usually jsut a "stem" starting point for the skeleton. Only those pieces
    /// referenced here in the mesh should actually be rendered. Any other pieces in the skeleton
    /// contain no vertices or polygons And have other purposes.
    vertex_pieces: Vec<(u16, u16)>,

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
    vertex_materials: Vec<(u16, u16)>,

    /// _Unknown_ - A collection of [MeshFragmentData9Entry]s
    data9: Vec<MeshFragmentData9Entry>,
}

impl Fragment for MeshFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], MeshFragment> {
        let (
            i,
            (
                flags,
                material_list_ref,
                fragment2,
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
                flags,
                material_list_ref,
                fragment2,
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

impl Fragment for MeshFragmentPolygonEntry {
    type T = Self;

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
}

#[derive(Debug)]
/// _Unknown_
struct MeshFragmentData9Entry {
    /// _Unknown_ - This seems to reference one of the vertex entries. This field
    /// only exists if `type_field` contains a value in the range 1-3.
    index1: Option<u16>,

    /// _Unknown_ - This seems to reference one of the vertex entries. This field is only valid if
    /// `type_field` contains 1. Otherwise, this field must contain 0.
    index2: Option<u16>,

    /// _Unknown_ - If `type_field` contains 4, then this field exists instead of `index1`
    /// and `index2`. [MeshFragmentData9Entry]s seem to be sorted by this value.
    offset: Option<f32>,

    /// _Unknown_ - It seems to only contain values in the range 0-2.
    param1: u16,

    /// _Unknown_ - It seems to control whether `index1`, `index2`, and `offset` exist. It can only
    /// contain values in the range 1-4. It looks like the [MeshFragmentData9Entry]s are broken up into
    /// blocks, where each block is terminated by an entry where `type_field` is 4.
    type_field: u16,
}

impl Fragment for MeshFragmentData9Entry {
    type T = Self;

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
}

#[derive(Debug)]
///
/// **Type ID:** 0x31
pub struct MaterialListFragment {
    /// _Unknown_ - Must contain 0.
    flags: u32,

    /// The number of fragment references this fragment contains.
    size1: u32,

    /// `size1` references to [MaterialFragment] fragments.
    pub fragments: Vec<FragmentRef<MaterialFragment>>,
}

impl Fragment for MaterialListFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialListFragment> {
        let (i, (flags, size1)) = tuple((le_u32, le_u32))(input)?;
        let (remaining, fragments) = count(fragment_ref, size1 as usize)(i)?;
        Ok((
            remaining,
            MaterialListFragment {
                flags,
                size1,
                fragments,
            },
        ))
    }
}

#[derive(Debug)]
///
/// **Type ID:** 0x30
pub struct MaterialFragment {
    /// Most flags are _unknown_, however:
    /// * bit 1 - If set then the `pair` field exists. This is usually set.
    flags: u32,

    /// Most flags are _unknown_, however:
    /// * bit 0 - It seems like this must be set if the texture is not transparent.
    /// * bit 1 - Set if the texture is masked (e.g. tree leaves).
    /// * bit 2 - Set if the texture is semi-transparent but not masked.
    /// * bit 3 - Set if the texture is masked and semi-transparent.
    /// * bit 4 Set if the texture is masked but not semi-transparent.
    /// * bit 31 - It seems like this must be set if the texture is not transparent.
    params1: u32,

    /// This typically contains 0x004E4E4E but has also bee known to contain 0xB2B2B2.
    /// Could this be an RGB reflectivity value?
    params2: u32,

    /// _Unknown_ - Usually contains 0.
    params3: (f32, f32),

    /// A reference to a [TextureReferenceFragment] fragment.
    pub reference: FragmentRef<TextureReferenceFragment>,

    /// _Unknown_ - This only exists if bit 1 of flags is set. Both fields usually contain 0.
    pair: Option<(u32, f32)>,
}

impl Fragment for MaterialFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialFragment> {
        let (i, (flags, params1, params2, params3, reference)) = tuple((
            le_u32,
            le_u32,
            le_u32,
            tuple((le_f32, le_f32)),
            fragment_ref,
        ))(input)?;

        let (remaining, pair) = if flags & 0x2 == 0x2 {
            tuple((le_u32, le_f32))(i).map(|(rem, p)| (rem, Some(p)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            MaterialFragment {
                flags,
                params1,
                params2,
                params3,
                reference,
                pair,
            },
        ))
    }
}

#[derive(Debug)]
/// A reference to a [TextureFragment] fragment.
///
/// **Type ID:** 0x05
pub struct TextureReferenceFragment {
    /// The [TextureFragment] reference.
    pub reference: FragmentRef<TextureFragment>,

    /// _Unknown_ - Seems to always contain 0x50.
    flags: u32,
}

impl Fragment for TextureReferenceFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureReferenceFragment> {
        let (remaining, (reference, flags)) = tuple((fragment_ref, le_u32))(input)?;
        Ok((remaining, TextureReferenceFragment { reference, flags }))
    }
}

#[derive(Debug)]
/// This fragment represents an entire texture rather than merely a bitmap used by that
/// texture. The conceptual difference from [TextureImagesFragment] fragments is that textures
/// may be animated; the [TextureFragment] fragment represents the entire texture
/// including all bitmaps that it uses whereas a [TextureImagesFragment] fragment would
/// represent only a single bitmap in the animated sequence.
///
/// **Type ID:** 0x04
pub struct TextureFragment {
    /// Most flags are _unknown_ however:
    /// * bit 3 - If set texture is animated (has more than one [TextureImagesFragment] reference.
    /// This also means that a `params1` field exists.
    /// * bit 4 - If set a `params2` field exists. This _seems_ to always be set.
    flags: u32,

    /// The number of [TextureImagesFragment] references.
    size: u32,

    /// _Unknown_ - Only present if bit 3 of `flags` is set.
    params1: Option<u32>,

    /// _Unknown_ - Only present if bit 4 of `flags` is set.
    params2: Option<u32>,

    /// One or more references to [TextureImagesFragment] fragments. For most textures this will
    /// be a single reference but animated textures will reference multiple.
    pub references: Vec<FragmentRef<TextureImagesFragment>>,
}

impl Fragment for TextureFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureFragment> {
        let (i, (flags, size)) = tuple((le_u32, le_u32))(input)?;

        // TODO: Do these fields even really exist?
        let params1 = None;
        let params2 = None;
        let (remaining, references) = count(fragment_ref, size as usize)(i)?;

        Ok((
            remaining,
            TextureFragment {
                flags,
                size,
                params1,
                params2,
                references,
            },
        ))
    }
}

#[derive(Debug)]
/// This fragment references one or more texture filenames. So far all known textures
/// reference a single filename.
///
/// **Type ID:** 0x03
pub struct TextureImagesFragment {
    /// Contains the number of texture filenames in this fragment. Again, this appears
    /// to always be 1.
    size1: u32,

    /// Bitmap filename entries
    pub entries: Vec<TextureImagesFragmentEntry>,
}

impl Fragment for TextureImagesFragment {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureImagesFragment> {
        let (i, size1) = le_u32(input)?;
        // TODO: This is hardcoded to one entry, is this all we need?
        let (remaining, entries) = count(TextureImagesFragmentEntry::parse, 1 as usize)(i)?;
        Ok((remaining, TextureImagesFragment { size1, entries }))
    }
}

#[derive(Debug)]
/// Bitmap filename entries within the [TextureImagesFragment] fragment.
pub struct TextureImagesFragmentEntry {
    /// The length of the filename in bytes.
    name_length: u16,

    /// The encoded filename. See [string hash encoding].
    ///
    /// The client apparently looks for certain filenames and substitutes built-in
    /// textures in their place. When using an animated fire texture where the names
    /// are fire1.bmp, fire2.bmp, fire3.bmp and fire4.bmp, respectively, the client always
    /// uses its built-in fire textures instead. This only happens when the textures are
    /// used by a placeable object and not when the textures are in the main zone file.
    /// It is unknown whether the substitution depends on the presence and exact order
    /// of all four textures.
    pub file_name: String,
}

impl Fragment for TextureImagesFragmentEntry {
    type T = Self;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureImagesFragmentEntry> {
        let (i, name_length) = le_u16(input)?;
        let (remaining, file_name) = count(le_u8, name_length as usize)(i)?;
        Ok((
            remaining,
            TextureImagesFragmentEntry {
                name_length,
                file_name: decode_string(&file_name),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsp_tree() {
        let data: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, // Size1
            0x00, 0x00, 0x80, 0xbf, // Entry1NormalX
            0x00, 0x00, 0x00, 0x00, // Entry1NormalY
            0x00, 0x00, 0x00, 0x00, // Entry1NormalZ
            0xea, 0xe4, 0x3b, 0xc3, // Entry1SplitDistance
            0x00, 0x00, 0x00, 0x00, // Entry1RegionID
            0x02, 0x00, 0x00, 0x00, // Entry1Node1
            0xcb, 0x09, 0x00, 0x00, // Entry1Node2
            0x00, 0x00, 0x80, 0xbf, // Entry2NormalX
            0x00, 0x00, 0x00, 0x00, // Entry2NormalY
            0x00, 0x00, 0x00, 0x00, // Entry2NormalZ
            0xea, 0xe4, 0x3b, 0xc3, // Entry2SplitDistance
            0x00, 0x00, 0x00, 0x00, // Entry2RegionID
            0x02, 0x00, 0x00, 0x00, // Entry2Node1
            0xcb, 0x09, 0x00, 0x00, // Entry2Node2
        ];
        let (_, result) = BspTreeFragment::parse(&data).unwrap();

        assert_eq!(result.size1, 2);
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_bsp_tree_entry() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0x80, 0xbf, // NormalX
            0x00, 0x00, 0x00, 0x00, // NormalY
            0x00, 0x00, 0x00, 0x00, // NormalZ
            0xea, 0xe4, 0x3b, 0xc3, // SplitDistance
            0x00, 0x00, 0x00, 0x00, // RegionID
            0x02, 0x00, 0x00, 0x00, // Node1
            0xcb, 0x09, 0x00, 0x00, // Node2
        ];
        let (_, result) = BspTreeFragmentEntry::parse(&data).unwrap();

        assert_eq!(result.normal, (-1.0, 0.0, 0.0));
        assert_eq!(result.split_distance, -187.8942);
        assert_eq!(result.region, 0);
        assert_eq!(result.nodes, (2, 2507));
    }

    #[test]
    fn test_bsp_region() {
        let data: Vec<u8> = vec![
            0x81, 0x01, 0x00, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // fragment1
            0x02, 0x00, 0x00, 0x00, // size1
            0x02, 0x00, 0x00, 0x00, // size2
            0x00, 0x00, 0x00, 0x00, // params1
            0x01, 0x00, 0x00, 0x00, // size3
            0x01, 0x00, 0x00, 0x00, // size4
            0x00, 0x00, 0x00, 0x00, // params2
            0x01, 0x00, 0x00, 0x00, // size5
            0x01, 0x00, 0x00, 0x00, // size6
            0x01, 0x02, // data1
            0x03, 0x04, // data2
            // data3
            0x00, 0x00, 0x00, 0x00, // data3.flags
            0x02, 0x00, 0x00, 0x00, // data3.size1
            0x01, 0x00, 0x00, 0x00, // data3.data1[0]
            0x02, 0x00, 0x00, 0x00, // data3.data1[1]
            // data4
            0x00, 0x00, 0x00, 0x00, // data4.flags
            0x01, 0x00, 0x00, 0x00, // data4.params1
            0x01, 0x00, 0x00, 0x00, // data4.type_field
            0x02, 0x00, 0x00, 0x00, // data4.name_size
            0x41, 0x42, // data4.name
            // data5
            0x01, 0x00, 0x00, 0x00, // data5.params1.0
            0x02, 0x00, 0x00, 0x00, // data5.params1.1
            0x03, 0x00, 0x00, 0x00, // data5.params1.2
            0x04, 0x00, 0x00, 0x00, // data5.params2
            0x05, 0x00, 0x00, 0x00, // data5.params3
            0x06, 0x00, 0x00, 0x00, // data5.params4
            0x07, 0x00, 0x00, 0x00, // data5.params5
            // data6
            0x02, 0x00, // data6.size1
            0x01, 0x02, // data6.data
            0x02, 0x00, 0x00, 0x00, // size7
            0x00, 0x00, 0x00, 0x00, // name7
            0x00, 0x00, 0x00, 0x00, // name7
            0x00, 0x00, 0x00, 0x00, // name7
            0x00, 0x00, 0x00, 0x00, // fragment2
            0x01, 0x00, 0x00, 0x00, // fragment3
        ];
        let (_, result) = BspRegionFragment::parse(&data).unwrap();

        assert_eq!(result.flags, 0x181);
        assert_eq!(result.fragment1, 1);
        assert_eq!(result.size1, 2);
        assert_eq!(result.size2, 2);
        assert_eq!(result.params1, 0);
        assert_eq!(result.size3, 1);
        assert_eq!(result.size4, 1);
        assert_eq!(result.params2, 0);
        assert_eq!(result.size5, 1);
        assert_eq!(result.size6, 1);
        assert_eq!(result.data1, vec![0x01, 0x02]);
        assert_eq!(result.data2, vec![0x03, 0x04]);
        assert_eq!(result.data3.len(), 1);
        assert_eq!(result.data4.len(), 1);
        assert_eq!(result.data5.len(), 1);
        assert_eq!(result.data6.len(), 1);
        assert_eq!(result.size7, 2);
        //assert_eq!(result.name7, "AB");
        assert_eq!(result.fragment2, 0x00);
        assert_eq!(result.fragment3, Some(0x01));
    }

    #[test]
    fn test_bsp_region_data3_entry_with_params() {
        let data: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, // flags
            0x02, 0x00, 0x00, 0x00, // size1
            0x01, 0x00, 0x00, 0x00, // data1[0]
            0x02, 0x00, 0x00, 0x00, // data1[1]
            0x01, 0x00, 0x00, 0x00, // params1.0
            0x02, 0x00, 0x00, 0x00, // params1.1
            0x03, 0x00, 0x00, 0x00, // params1.2
            0x01, 0x00, 0x00, 0x00, // params2
        ];
        let (_, result) = BspRegionFragmentData3Entry::parse(&data).unwrap();

        assert_eq!(result.flags, 0x02);
        assert_eq!(result.size1, 2);
        assert_eq!(result.data1.len(), 2);
        assert_eq!(result.data1[0], 1);
        assert_eq!(result.data1[1], 2);
        assert_eq!(result.params1, Some((1, 2, 3)));
        assert_eq!(result.params2, Some(1));
    }

    #[test]
    fn test_bsp_region_data3_entry_without_params() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, // flags
            0x02, 0x00, 0x00, 0x00, // size1
            0x01, 0x00, 0x00, 0x00, // data1[0]
            0x02, 0x00, 0x00, 0x00, // data1[1]
        ];
        let (_, result) = BspRegionFragmentData3Entry::parse(&data).unwrap();

        assert_eq!(result.flags, 0x00);
        assert_eq!(result.size1, 2);
        assert_eq!(result.data1.len(), 2);
        assert_eq!(result.data1[0], 1);
        assert_eq!(result.data1[1], 2);
        assert_eq!(result.params1, None);
        assert_eq!(result.params2, None);
    }

    #[test]
    fn test_bsp_region_data4_entry_without_params2() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // params1
            0x01, 0x00, 0x00, 0x00, // type_field
            0x02, 0x00, 0x00, 0x00, // name_size
            0x41, 0x42, // name
        ];
        let (_, result) = BspRegionFragmentData4Entry::parse(&data).unwrap();

        assert_eq!(result.flags, 0x00);
        assert_eq!(result.params1, 1);
        assert_eq!(result.type_field, 1);
        assert_eq!(result.params2a, None);
        assert_eq!(result.params2b, None);
        assert_eq!(result.name_size, 2);
        assert_eq!(result.name, "AB");
    }

    #[test]
    fn test_bsp_region_data4_entry_with_params2() {
        let data: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // params1
            0x0a, 0x00, 0x00, 0x00, // type_field
            0x01, 0x00, 0x00, 0x00, // params2a
            0x02, 0x00, 0x00, 0x00, // params2b
            0x02, 0x00, 0x00, 0x00, // name_size
            0x41, 0x42, // name
        ];
        let (_, result) = BspRegionFragmentData4Entry::parse(&data).unwrap();

        assert_eq!(result.flags, 0x00);
        assert_eq!(result.params1, 1);
        assert_eq!(result.type_field, 10);
        assert_eq!(result.params2a, Some(1));
        assert_eq!(result.params2b, Some(2));
        assert_eq!(result.name_size, 2);
        assert_eq!(result.name, "AB");
    }

    #[test]
    fn test_bsp_region_data5_entry() {
        let data: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // params1.0
            0x02, 0x00, 0x00, 0x00, // params1.1
            0x03, 0x00, 0x00, 0x00, // params1.2
            0x04, 0x00, 0x00, 0x00, // params2
            0x05, 0x00, 0x00, 0x00, // params3
            0x06, 0x00, 0x00, 0x00, // params4
            0x07, 0x00, 0x00, 0x00, // params5
        ];
        let (_, result) = BspRegionFragmentData5Entry::parse(&data).unwrap();

        assert_eq!(result.params1, (1, 2, 3));
        assert_eq!(result.params2, 4);
        assert_eq!(result.params3, 5);
        assert_eq!(result.params4, 6);
        assert_eq!(result.params5, 7);
    }

    #[test]
    fn test_bsp_region_data6_entry() {
        let data: Vec<u8> = vec![
            0x02, 0x00, // size1
            0x01, 0x02, // data
        ];
        let (_, result) = BspRegionFragmentData6Entry::parse(&data).unwrap();

        assert_eq!(result.size1, 2);
        assert_eq!(result.data[0], 1);
        assert_eq!(result.data[1], 2);
    }

    #[test]
    fn test_mesh() {
        let data: Vec<u8> = vec![
            0x03, 0x80, 0x01, 0x00, // flags
            0x01, 0x00, 0x00, 0x00, // fragment1
            0x02, 0x00, 0x00, 0x00, // fragment2
            0x03, 0x00, 0x00, 0x00, // fragment3
            0x04, 0x00, 0x00, 0x00, // fragment4
            0x00, 0x00, 0x80, 0x3f, // center.0
            0x00, 0x00, 0x80, 0x3f, // center.1
            0x00, 0x00, 0x80, 0x3f, // center.2
            0x01, 0x00, 0x00, 0x00, // params2.0
            0x01, 0x00, 0x00, 0x00, // params2.1
            0x01, 0x00, 0x00, 0x00, // params2.2
            0x00, 0x00, 0x80, 0x3f, // max_distance
            0x00, 0x00, 0x80, 0x3f, // min.0
            0x00, 0x00, 0x80, 0x3f, // min.1
            0x00, 0x00, 0x80, 0x3f, // min.2
            0x00, 0x00, 0x80, 0x3f, // max.0
            0x00, 0x00, 0x80, 0x3f, // max.1
            0x00, 0x00, 0x80, 0x3f, // max.2
            0x02, 0x00, // vertex_count
            0x02, 0x00, // texture_coordinate_count
            0x02, 0x00, // normal_count
            0x02, 0x00, // color_count
            0x02, 0x00, // polygon_count
            0x02, 0x00, // vertex_piece_count
            0x02, 0x00, // polygon_texture_count
            0x02, 0x00, // vertex_texture_count
            0x01, 0x00, // size9
            0x01, 0x00, // scale
            0x01, 0x00, 0x01, 0x00, 0x01, 0x00, // vertices[0]
            0x02, 0x00, 0x02, 0x00, 0x02, 0x00, // vertices[1]
            0x01, 0x00, 0x01, 0x00, // texture_coordinates[0]
            0x02, 0x00, 0x02, 0x00, // texture_coordinates[1]
            0x01, 0x01, 0x01, // vertex_normals[0]
            0x02, 0x02, 0x02, // vertex_normals[1]
            0x01, 0x00, 0x00, 0x00, // vertex_colors[0]
            0x02, 0x00, 0x00, 0x00, // vertex_colors[1]
            0x01, 0x00, // polygons[0].flags
            0x01, 0x00, // polygons[0].vertex_indexes.0
            0x02, 0x00, // polygons[0].vertex_indexes.1
            0x03, 0x00, // polygons[0].vertex_indexes.2
            0x01, 0x00, // polygons[1].flags
            0x01, 0x00, // polygons[1].vertex_indexes.0
            0x02, 0x00, // polygons[1].vertex_indexes.1
            0x03, 0x00, // polygons[1].vertex_indexes.2
            0x01, 0x00, 0x01, 0x00, // vertex_pieces[0]
            0x02, 0x00, 0x02, 0x00, // vertex_pieces[1]
            0x01, 0x00, 0x01, 0x00, // polygon_textures[0]
            0x02, 0x00, 0x02, 0x00, // polygon_textures[1]
            0x01, 0x00, 0x01, 0x00, // vertex_textures[0]
            0x02, 0x00, 0x02, 0x00, // vertex_textures[1]
            0x01, 0x00, // data9[0].index1
            0x02, 0x00, // data9[0].index2
            0x00, 0x00, 0x80, 0x3f, // data9[0].offset
            0x01, 0x00, // data9[0].param1
            0x01, 0x00, // data9[0].type_field
        ];
        let (_, result) = MeshFragment::parse(&data).unwrap();

        assert_eq!(result.flags, 0x00018003);
        assert_eq!(result.fragment1, 1);
        assert_eq!(result.fragment2, 2);
        assert_eq!(result.fragment3, 3);
        assert_eq!(result.fragment4, 4);
        assert_eq!(result.center, (1f32, 1f32, 1f32));
        assert_eq!(result.params2, (1, 1, 1));
        assert_eq!(result.max_distance, 1f32);
        assert_eq!(result.min, (1f32, 1f32, 1f32));
        assert_eq!(result.max, (1f32, 1f32, 1f32));
        assert_eq!(result.vertex_count, 2);
        assert_eq!(result.texture_coordinate_count, 2);
        assert_eq!(result.normal_count, 2);
        assert_eq!(result.color_count, 2);
        assert_eq!(result.polygon_count, 2);
        assert_eq!(result.vertex_piece_count, 2);
        assert_eq!(result.polygon_texture_count, 2);
        assert_eq!(result.vertex_texture_count, 2);
        assert_eq!(result.size9, 1);
        assert_eq!(result.scale, 1);
        assert_eq!(result.vertices, vec![(1, 1, 1), (2, 2, 2)]);
        assert_eq!(result.texture_coordinates, vec![(1, 1), (2, 2)]);
        assert_eq!(result.vertex_normals, vec![(1, 1, 1), (2, 2, 2)]);
        assert_eq!(result.vertex_colors, vec![1, 2]);
        assert_eq!(result.polygons.len(), 2);
        assert_eq!(result.vertex_pieces, vec![(1, 1), (2, 2)]);
        assert_eq!(result.polygon_textures, vec![(1, 1), (2, 2)]);
        assert_eq!(result.vertex_textures, vec![(1, 1), (2, 2)]);
        assert_eq!(result.data9.len(), 1);
    }

    #[test]
    fn test_mesh_data9_entry() {
        let data: Vec<u8> = vec![
            0x01, 0x00, // index1
            0x02, 0x00, // index2
            0x00, 0x00, 0x80, 0x3f, // offset
            0x01, 0x00, // param1
            0x01, 0x00, // type_field
        ];
        let (_, result) = MeshFragmentData9Entry::parse(&data).unwrap();

        assert_eq!(result.index1, Some(1));
        assert_eq!(result.index2, Some(2));
        assert_eq!(result.offset, Some(1f32));
        assert_eq!(result.param1, 1);
        assert_eq!(result.type_field, 1);
    }

    #[test]
    fn test_mesh_polygon_entry() {
        let data: Vec<u8> = vec![
            0x01, 0x00, // flags
            0x01, 0x00, // vertex_indexes.0
            0x02, 0x00, // vertex_indexes.1
            0x03, 0x00, // vertex_indexes.2
        ];
        let (_, result) = MeshFragmentPolygonEntry::parse(&data).unwrap();

        assert_eq!(result.flags, 1);
        assert_eq!(result.vertex_indexes, (1, 2, 3));
    }
}
