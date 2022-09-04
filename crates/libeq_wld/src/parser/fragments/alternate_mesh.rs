use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, MaterialListFragment, StringReference, WResult};

use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_u16, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// This fragment is rarely seen. It is very similar to the 0x36 [MeshFragment].
/// I believe that this might have been the original type and was later replaced
/// by the 0x36 [MeshFragment]. I’ve only seen one example of this fragment so
/// far so the information here is uncertain.
///
/// **Type ID:** 0x2c
pub struct AlternateMeshFragment {
    pub name_reference: StringReference,

    /// Most fields are _unknown_. This usually contains 0x00001803.
    /// * bit  0 - If set then `center_x`, `center_y`, and `center_z` are valid.
    ///            Otherwise they must contain 0.
    /// * bit  1 - If set then `params2` is valid. Otherwise it must contain 0.
    /// * bit  9 - If set then the `size8` and `data8` entries exist.
    /// * bit 11 - If set then the `polygon_tex_count` field and `polygon_tex` entries exist.
    /// * bit 12 - If set then the `vertex_tex_count` field and `vertex_tex` entries exist.
    /// * bit 13 - If set then the `params_3[]` fields exist
    /// * bit 14 - If set then the `params_4[]` fields exist
    pub flags: u32,

    /// Tells how many vertices there are in the mesh. Normally this is three times the number
    /// of polygons, but this is by no means necessary as polygons can share vertices. However,
    /// sharing vertices degrades the ability to use vertex normals to make a mesh look
    /// more rounded (with shading).
    pub vertex_count: u32,

    /// Tells how many texture coordinate pairs there are in the mesh. This should equal the
    /// number of vertices in the mesh. Presumably this could contain zero if none of the
    /// polygons have textures mapped to them (but why would anyone do that?)
    pub tex_coords_count: u32,

    /// Tells how many vertex normal entries there are in the mesh. This should equal the number
    /// of vertices in the mesh. Presumably this could contain zero if vertices should use
    /// polygon normals instead, but I haven’t tried it (vertex normals are preferable anyway).
    pub normals_count: u32,

    /// Its purpose is unknown (though if the pattern with the 0x36 fragment holds then it
    /// should contain color information).
    pub size4: u32,

    /// The number of polygons in the mesh.
    pub polygon_count: u32,

    /// This seems to only be used when dealing with animated (mob) models.
    /// It determines the number of entries in `data6`.
    pub size6: u16,

    /// This seems to only be used when dealing with animated (mob) models. It tells how many
    /// VertexPiece entries there are. Vertices are grouped together by skeleton piece in this
    /// case and VertexPiece entries tell the client how many vertices are in each piece.
    /// It’s possible that there could be more pieces in the skeleton than are in the meshes
    /// it references. Extra pieces have no polygons or vertices and I suspect they are there
    /// to define attachment points for objects (e.g. weapons or shields).
    pub vertex_piece_count: i16,

    /// References a 0x31 [MaterialListFragment]. It tells the client which textures this mesh
    /// uses. For zone meshes, a single 0x31 fragment should be built that contains all the
    /// textures used in the entire zone. For placeable objects, there should be a 0x31
    /// fragment that references only those textures used in that particular object.
    //pub fragment1: FragmentRef<MaterialListFragment>,
    pub fragment1: u32,

    /// _Unknown_
    pub fragment2: u32,

    /// _Unknown_
    pub fragment3: u32,

    /// This seems to define the center of the model and is used for positioning (I think).
    pub center: (f32, f32, f32),

    /// _Unknown_ FIXME: Could be float
    pub params2: u32,

    /// There are `vertex_count` of these.
    pub vertices: Vec<(f32, f32, f32)>,

    /// There are `tex_coords_count` of these.
    pub texture_coords: Vec<(f32, f32)>,

    /// There are `normals_count` of these
    pub normals: Vec<(f32, f32, f32)>,

    /// _Unknown_ - There are `size4` of these.
    pub data4: Vec<u32>,

    /// _Unknown_ - There are `polygon_count` of these.
    /// First tuple value seems to be flags, usually contains 0x004b for polygons.
    /// Second tuple values are usually zero. Their purpose is _unknown_.
    pub polygons: Vec<AlternateMeshFragmentPolygonEntry>,

    /// There are `size6` of these.
    pub data6: Vec<AlternateMeshFragmentData6Entry>,

    /// The first element of the tuple is the number of vertices in a skeleton piece.
    ///
    /// The second element of the tuple is the index of the piece according to the
    /// [SkeletonTrackSet] fragment. The very first piece (index 0) is usually not referenced here
    /// as it is usually jsut a "stem" starting point for the skeleton. Only those pieces
    /// referenced here in the mesh should actually be rendered. Any other pieces in the skeleton
    /// contain no vertices or polygons And have other purposes.
    pub vertex_pieces: Vec<(u16, u16)>,

    /// _Unknown_ - This only exists if bit 9 of `flags` is set.
    pub size8: Option<u32>,

    /// _Unknown_ - This only exists if bit 9 of `flags` is set. There are `size8` of these.
    pub data8: Option<Vec<u32>>,

    /// _Unknown_
    pub params4: Vec<u16>,
    
    /// _Unknown_ - There are 'fragment1' of these if Bit 0 is unset
    pub data9: Vec<(u16,u16)>,

    /// Tells how many PolygonTex entries there are. Polygons are grouped together by texture and PolygonTex entries tell the client how many polygons there are that use a particular texture. This field only exists if bit 11 of Flags is 1.
    pub polygontex_count: Option<u32>,
    
    /// PolygonTex entries (there are PolygonTexCount of these)
    pub polygontex_entries: Option<Vec<(u16,u16)>>,

    /// The number of `vertex_materials` entries there are. Polygons are grouped together by
    /// material and `vertex_material` entries telling the client how many polygons there are
    /// that use a particular material. This field only exists if bit 12 of `flags` is set.
    pub vertex_material_count: Option<u32>,

    /// The first element of the tuple is the number of vertices that use the same
    /// material. Vertex materials, like polygons, are sorted by material index so
    /// that vertices that use the same material are together.
    ///
    /// The second element of the tuple is the index of the material that the
    /// vertices use, according to the [MaterialListFragment] fragment that this fragment
    /// references. This field only exists if bit 12 of `flags` is set.
    ///
    /// The rest are _Unknown_
    ///
    /// There are 'vertex_material_count` of these.
    pub vertex_materials: Option<Vec<(u16, u16)>>,
    
    /// its purpose is unknown. This field only exists if bit 13 of Flags is 1.
    pub params3: Option<(u32,u32,u32)>,

    /// its purpose is unknown. This field only exists if bit 14 of Flags is 1.
    pub params5: Option<(u32,u32,u32,u32,u32,u32)>


}

impl FragmentParser for AlternateMeshFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2c;
    const TYPE_NAME: &'static str = "AlternateMesh";

    fn parse(input: &[u8]) -> WResult<AlternateMeshFragment> {
        let (
            i,
            (
                name_reference,
                flags,
                vertex_count,
                tex_coords_count,
                normals_count,
                size4,
                polygon_count,
                size6,
                vertex_piece_count,
                fragment1,
                fragment2,
                fragment3,
                center,
                params2,
            ),
        ) = tuple((
            StringReference::parse,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u32,
            le_u16,
            le_i16,
            le_u32,
            le_u32,
            le_u32,
            tuple((le_f32, le_f32, le_f32)),
            le_u32,
        ))(input)?;

        let (i, (vertices, texture_coords, normals, data4, polygons, data6, vertex_pieces)) =
        tuple((
            count(tuple((le_f32, le_f32, le_f32)), vertex_count as usize),
            count(tuple((le_f32, le_f32)), tex_coords_count as usize),
            count(tuple((le_f32, le_f32, le_f32)), normals_count as usize),
            count(le_u32, size4 as usize),
            count(
                AlternateMeshFragmentPolygonEntry::parse,
                polygon_count as usize,
            ),
            count(AlternateMeshFragmentData6Entry::parse, size6 as usize),
            count(tuple((le_u16, le_u16)), vertex_piece_count as usize)
        ))(i)?;

        let (i, size8) = if flags & 0x200 == 0x200 { // Bit 9 is set
            le_u32(i).map(|(i, size8)| (i, Some(size8)))?
        } else {
            (i, None)
        };

        let (i, data8) = if flags & 0x200 == 0x200 { // Bit 9 is set
            count(le_u32, size8.unwrap() as usize)(i).map(|(i, data8)| (i, Some(data8)))?
        } else {
            (i, None)
        };

        let (i, params4) = count(le_u16, 4)(i)?;

        let (i, data9) = if flags & 0x01 != 0x01 { // Bit 0 is unset
            count(tuple((le_u16, le_u16)), fragment1 as usize)(i)?
        } else {
            (i, vec![])
        };

        let (i, polygontex_count) = if flags & 0x800 == 0x800 { // Bit 11 set
            le_u32(i).map(|(i, polygontex_count)| (i, Some(polygontex_count)))?
        } else {
            (i, None)
        };

        let (i, polygontex_entries) = if flags & 0x800 == 0x800 { // Bit 11 set
            count(tuple((le_u16, le_u16)),  polygontex_count.unwrap() as usize)(i).map(|(i, polygontex_entries)| (i, Some(polygontex_entries)))?
        } else {
            (i, None)
        };

        let (i, vertex_material_count) = if flags & 0x1000 == 0x1000 { // Bit 12 set
            le_u32(i).map(|(i, vertex_material_count)| (i, Some(vertex_material_count)))?
        } else {
            (i, None)
        };

        let (i, vertex_materials) = if flags & 0x1000 == 0x1000 { // Bit 12 set
            count(tuple((le_u16, le_u16)),  vertex_material_count.unwrap() as usize)(i).map(|(i, vertex_materials)| (i, Some(vertex_materials)))?
        } else {
            (i, None)
        };
        
        let (i, params3) = if flags & 0x2000 == 0x2000 { // Bit 13 set
            tuple((le_u32, le_u32, le_u32))(i).map(|(i, params3)| (i, Some(params3)))?
        } else {
            (i, None)
        };

        let (i, params5) = if flags & 0x4000 == 0x4000 { // Bit 14 set
            tuple((le_u32, le_u32, le_u32, le_u32, le_u32, le_u32))(i).map(|(i, params5)| (i, Some(params5)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            AlternateMeshFragment {
                name_reference,
                flags,
                vertex_count,
                tex_coords_count,
                normals_count,
                size4,
                polygon_count,
                size6,
                vertex_piece_count,
                fragment1,
                fragment2,
                fragment3,
                center,
                params2,
                vertices,
                texture_coords,
                normals,
                data4,
                polygons,
                data6,
                vertex_pieces,
                size8,
                data8,
                params4,
                data9,
                polygontex_count,
                polygontex_entries,
                vertex_material_count,
                vertex_materials,
                params3,
                params5
            },
        ))
    }
}

impl Fragment for AlternateMeshFragment {
    fn into_bytes(&self) -> Vec<u8> {
        let vertices = self
            .vertices
            .iter()
            .flat_map(|v| vec![v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()])
            .flatten()
            .collect::<Vec<_>>();
        let texture_coords = self
            .texture_coords
            .iter()
            .flat_map(|v| vec![v.0.to_le_bytes(), v.1.to_le_bytes()])
            .flatten()
            .collect::<Vec<_>>();
        let normals = self
            .normals
            .iter()
            .flat_map(|v| vec![v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()])
            .flatten()
            .collect::<Vec<_>>();
        let polygons = self
            .polygons
            .iter()
            .flat_map(|p| p.into_bytes())
            .collect::<Vec<_>>();
        let data6 = self
            .data6
            .iter()
            .flat_map(|d| d.into_bytes())
            .collect::<Vec<_>>();

        let data8 = self
            .data8
            .as_ref()
            .map_or(vec![], |i| i.iter().flat_map(|d| d.to_le_bytes()).collect::<Vec<_>>());

        let params4 = self
            .params4
            .iter()
            .flat_map(|d| d.to_le_bytes())
            .collect::<Vec<_>>();

        let vertex_pieces = self
            .vertex_pieces
            .iter()
            .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes()].concat())
            .collect::<Vec<_>>();

        let polygontex_entries = self
            .polygontex_entries
            .as_ref()
            .map_or(vec![], |i| i.iter()
            .flat_map(|v| {
                [
                    &v.0.to_le_bytes()[..],
                    &v.1.to_le_bytes()[..],
                ]
                .concat()
            })
            .collect::<Vec<_>>());

        let vertex_materials = self
            .vertex_materials
            .as_ref()
            .map_or(vec![], |i| i.iter()
            .flat_map(|v| {
                [
                    &v.0.to_le_bytes()[..],
                    &v.1.to_le_bytes()[..],
                ]
                .concat()
            })
            .collect::<Vec<_>>());

        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.vertex_count.to_le_bytes()[..],
            &self.tex_coords_count.to_le_bytes()[..],
            &self.normals_count.to_le_bytes()[..],
            &self.size4.to_le_bytes()[..],
            &self.polygon_count.to_le_bytes()[..],
            &self.size6.to_le_bytes()[..],
            &self.vertex_piece_count.to_le_bytes()[..],
            &self.fragment1.to_le_bytes()[..],
            &self.fragment2.to_le_bytes()[..],
            &self.fragment3.to_le_bytes()[..],
            &self.center.0.to_le_bytes()[..],
            &self.center.1.to_le_bytes()[..],
            &self.center.2.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &vertices[..],
            &texture_coords[..],
            &normals[..],
            &self
                .data4
                .iter()
                .flat_map(|d| d.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &polygons[..],
            &data6[..],
            &vertex_pieces[..],
            &self.size8.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &data8[..],
            &params4,
            &self
                .data9
                .iter()
                .flat_map(|d| [d.0.to_le_bytes(), d.1.to_le_bytes()].concat() )
                .collect::<Vec<_>>()[..],
            &self.polygontex_count.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &polygontex_entries[..],
            &self.vertex_material_count.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &vertex_materials[..],
            &self.params3.map_or(vec![], |p| {
                [p.0.to_le_bytes(), p.1.to_le_bytes(), p.2.to_le_bytes()].concat()
            })[..],
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
#[derive(Debug)]
/// Represents a polygon within a [AlternativeMeshFragment].
pub struct AlternateMeshFragmentPolygonEntry {
    /// This usually contains 0x004b for polygons.
    pub flags: u16,

    /// _Unknown_ - Usually contains zeros.
    pub data: (u16, u16, u16, u16),

    /// An index for each of the polygon's vertex coordinates (idx1, idx2, idx3).
    pub vertex_indexes: (u16, u16, u16),
}

impl AlternateMeshFragmentPolygonEntry {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.flags.to_le_bytes()[..],
            &self.data.0.to_le_bytes()[..],
            &self.data.1.to_le_bytes()[..],
            &self.data.2.to_le_bytes()[..],
            &self.data.3.to_le_bytes()[..],
            &self.vertex_indexes.0.to_le_bytes()[..],
            &self.vertex_indexes.1.to_le_bytes()[..],
            &self.vertex_indexes.2.to_le_bytes()[..],
        ]
        .concat()
    }
}

impl AlternateMeshFragmentPolygonEntry {
    fn parse(input: &[u8]) -> WResult<AlternateMeshFragmentPolygonEntry> {
        let (remaining, (flags, data, vertex_indexes)) = tuple((
            le_u16,
            tuple((le_u16, le_u16, le_u16, le_u16)),
            tuple((le_u16, le_u16, le_u16)),
        ))(input)?;
        Ok((
            remaining,
            AlternateMeshFragmentPolygonEntry {
                flags,
                data,
                vertex_indexes,
            },
        ))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AlternateMeshFragmentData6Entry {
    /// This seems to reference one of the vertex entries. This field only exists if `_type`
    /// contains a value in the range 1 to 3.
    pub vertex_index: Option<u32>,

    /// _Unknown_
    /// If `_type` contains 4 then this field exists instead of `vertex_index`. Data6 entries seem
    /// to be sorted by this value
    pub offset: Option<f32>,

    /// _Unknown_ - Seems to only contain values in the range 0 to 2.
    pub param1: u16,

    /// _Unknown_
    pub param2: u16,

    /// _Unknown_ - It seems to control whether VertexIndex1, VertexIndex2, and Offset exist.
    /// It can only contain values in the range 1 to 4. It looks like the Data9 entries are broken
    /// up into blocks, where each block is terminated by an entry where Data9Type is 4.
    pub type_field: u32,
}

impl AlternateMeshFragmentData6Entry {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.vertex_index.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &self.offset.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &self.param1.to_le_bytes()[..],
            &self.param2.to_le_bytes()[..],
            &self.type_field.to_le_bytes()[..],
        ]
        .concat()
    }
}

impl AlternateMeshFragmentData6Entry {
    fn parse(input: &[u8]) -> WResult<AlternateMeshFragmentData6Entry> {
        let unknown_data = &input[0..4];
        let input = &input[4..];

        let (remaining, (param1, param2, type_field)) =
            tuple((le_u16, le_u16, le_u32))(input)?;
        
        let (unknown_data, offset) = if type_field == 4 {
            le_f32(unknown_data).map(|(i, offset)| (i, Some(offset)))?
        } else {
            (unknown_data, None)
        };

        let (_, vertex_index) = if type_field != 4 {
            le_u32(unknown_data).map(|(i, vertex_index)| (i, Some(vertex_index)))?
        } else {
            (unknown_data, None)
        };


        Ok((
            remaining,
            AlternateMeshFragmentData6Entry {
                vertex_index,
                offset,
                param1,
                param2,
                type_field
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0005-0x2c.frag")[..];
        let frag = AlternateMeshFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-44));
        assert_eq!(frag.flags, 0x1802);
        assert_eq!(frag.vertex_count, 29);
        assert_eq!(frag.tex_coords_count, 29);
        assert_eq!(frag.normals_count, 29);
        assert_eq!(frag.size4, 0);
        assert_eq!(frag.polygon_count, 12);
        assert_eq!(frag.size6, 64);
        assert_eq!(frag.vertex_piece_count, 0);
        assert_eq!(frag.fragment1, 1);
        assert_eq!(frag.fragment2, 5);
        assert_eq!(frag.fragment3, 0);
        assert_eq!(frag.center, (0.0, 0.0, 0.0));
        assert_eq!(frag.params2, 0);
        assert_eq!(frag.vertices.len(), frag.vertex_count as usize);
        assert_eq!(frag.vertices[0], (0.0, 12.247571, -5.070387));
        assert_eq!(frag.texture_coords.len(), frag.tex_coords_count as usize);
        assert_eq!(frag.texture_coords[0], (-0.07326539, -4.9999995)); // FIXME: This does not look like a valid UV coordinate.  It should be between 0 and 1.
        assert_eq!(frag.normals.len(), frag.normals_count as usize);
        assert_eq!(frag.normals[0], (0.5, 0.5, 0.0));
        assert_eq!(frag.data4.len(), frag.size4 as usize);
        assert_eq!(frag.polygons.len(), frag.polygon_count as usize);
        assert_eq!(frag.polygons[1].flags, 0);
        assert_eq!(frag.polygons[0].data, (46010, 0, 49024, 75));
        assert_eq!(frag.polygons[0].vertex_indexes, (0, 0, 0));
        assert_eq!(frag.data6.len(), frag.size6 as usize);
        assert_eq!(frag.vertex_pieces.len(), frag.vertex_piece_count as usize);
        assert_eq!(frag.size8, None);
        assert_eq!(frag.data8, None);
        assert_eq!(frag.data9.len(), 1);
        assert_eq!(frag.polygontex_count, Some(1));
        assert_eq!(frag.polygontex_entries, Some(vec![(12, 0)]));
        assert_eq!(frag.vertex_material_count, Some(1));
        assert_eq!(frag.vertex_materials, Some(vec![(29, 0)]));
    }

    #[test]
    fn it_parses_with_bit14() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip_beta/0567-0x2c.frag")[..];
        let frag = AlternateMeshFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-6091));
        assert_eq!(frag.flags, 0x5803);
        assert_eq!(frag.vertex_count, 56);
        assert_eq!(frag.tex_coords_count, 56);
        assert_eq!(frag.normals_count, 56);
        assert_eq!(frag.size4, 0);
        assert_eq!(frag.polygon_count, 28);
        assert_eq!(frag.size6, 0);
        assert_eq!(frag.vertex_piece_count, 0);
        assert_eq!(frag.fragment1, 0);
        assert_eq!(frag.fragment2, 567);
        assert_eq!(frag.fragment3, 0);
        assert_eq!(frag.center, (0.0, 0.0, 2.2031567));
        assert_eq!(frag.params2, 3141746767); // FIXME: Doesn't seem like this should be a u32.  Could be a float.
        assert_eq!(frag.vertices.len(), frag.vertex_count as usize);
        assert_eq!(frag.texture_coords.len(), frag.tex_coords_count as usize);
        assert_eq!(frag.normals.len(), frag.normals_count as usize);
        assert_eq!(frag.data4.len(), frag.size4 as usize);
        assert_eq!(frag.polygons.len(), frag.polygon_count as usize);
        assert_eq!(frag.data6.len(), frag.size6 as usize);
        assert_eq!(frag.vertex_pieces.len(), frag.vertex_piece_count as usize);
        assert_eq!(frag.size8, None);
        assert_eq!(frag.data8, None);
        assert_eq!(frag.data9.len(), 0);
        assert_eq!(frag.polygontex_count, Some(2));
        assert_eq!(frag.vertex_material_count, Some(2));
        assert_eq!(frag.params5.unwrap().0, 3224882372); // FIXME: This is probably either a float or color 
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0005-0x2c.frag")[..];
        let frag = AlternateMeshFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
