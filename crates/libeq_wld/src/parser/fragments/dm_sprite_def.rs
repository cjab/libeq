use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, MaterialPalette, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_u16, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// This fragment is rarely seen. It is very similar to the 0x36 [DmSpriteDef2].
/// I believe that this might have been the original type and was later replaced
/// by the 0x36 [DmSpriteDef2]. I’ve only seen one example of this fragment so
/// far so the information here is uncertain.
///
/// **Type ID:** 0x2c
pub struct DmSpriteDef {
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
    /// of faces, but this is by no means necessary as faces can share vertices. However,
    /// sharing vertices degrades the ability to use vertex vertex_normals to make a mesh look
    /// more rounded (with shading).
    pub vertex_count: u32,

    /// Tells how many texture coordinate pairs there are in the mesh. This should equal the
    /// number of vertices in the mesh. Presumably this could contain zero if none of the
    /// faces have textures mapped to them (but why would anyone do that?)
    pub texture_coordinate_count: u32,

    /// Tells how many vertex normal entries there are in the mesh. This should equal the number
    /// of vertices in the mesh. Presumably this could contain zero if vertices should use
    /// polygon vertex_normals instead, but I haven’t tried it (vertex vertex_normals are preferable anyway).
    pub normal_count: u32,

    /// Its purpose is unknown (though if the pattern with the 0x36 fragment holds then it
    /// should contain color information).
    pub color_count: u32,

    /// The number of faces in the mesh.
    pub face_count: u32,

    /// This seems to only be used when dealing with animated (mob) models.
    /// It determines the number of entries in `meshops`.
    pub meshop_count: u16,

    pub fragment1: i16,

    /// This seems to only be used when dealing with animated (mob) models. It tells how many
    /// VertexPiece entries there are. Vertices are grouped together by skeleton piece in this
    /// case and VertexPiece entries tell the client how many vertices are in each piece.
    /// It’s possible that there could be more pieces in the skeleton than are in the meshes
    /// it references. Extra pieces have no faces or vertices and I suspect they are there
    /// to define attachment points for objects (e.g. weapons or shields).
    pub skin_assignment_group_count: u32,

    /// References a 0x31 [MaterialPalette]. It tells the client which textures this mesh
    /// uses. For zone meshes, a single 0x31 fragment should be built that contains all the
    /// textures used in the entire zone. For placeable objects, there should be a 0x31
    /// fragment that references only those textures used in that particular object.
    pub material_list_ref: FragmentRef<MaterialPalette>,

    /// _Unknown_
    pub fragment3: u32,

    /// This seems to define the center of the model and is used for positioning (I think).
    pub center: (f32, f32, f32),

    /// _Unknown_
    pub params1: (f32, f32, f32),

    /// There are `vertex_count` of these.
    pub vertices: Vec<(f32, f32, f32)>,

    /// There are `texture_coordinate_count` of these.
    pub texture_coordinates: Vec<(f32, f32)>,

    /// There are `normal_count` of these
    pub vertex_normals: Vec<(f32, f32, f32)>,

    /// There are `color_count` of these.
    pub vertex_colors: Vec<u32>,

    /// _Unknown_ - There are `face_count` of these.
    /// First tuple value seems to be flags, usually contains 0x004b for faces.
    /// Second tuple values are usually zero. Their purpose is _unknown_.
    pub faces: Vec<DmSpriteDefFaceEntry>,

    /// There are `meshop_count` of these.
    pub meshops: Vec<DmSpriteDefMeshopEntry>,

    /// The first element of the tuple is the number of vertices in a skeleton piece.
    ///
    /// The second element of the tuple is the index of the piece according to the
    /// [HierarchicalSpriteDef] fragment. The very first piece (index 0) is usually not referenced here
    /// as it is usually jsut a "stem" starting point for the skeleton. Only those pieces
    /// referenced here in the mesh should actually be rendered. Any other pieces in the skeleton
    /// contain no vertices or faces And have other purposes.
    pub skin_assignment_groups: Vec<(u16, u16)>,

    /// _Unknown_ - This only exists if bit 9 of `flags` is set.
    pub size8: Option<u32>,

    /// _Unknown_ - This only exists if bit 9 of `flags` is set. There are `size8` of these.
    pub data8: Option<Vec<u32>>,

    /// Tells how many PolygonTex entries there are. faces are grouped together by texture and PolygonTex entries tell the client how many faces there are that use a particular texture. This field only exists if bit 11 of Flags is 1.
    pub face_material_group_count: Option<u32>,

    /// PolygonTex entries (there are PolygonTexCount of these)
    pub face_material_groups: Option<Vec<(u16, u16)>>,

    /// The number of `vertex_material_groups` entries there are. faces are grouped together by
    /// material and `vertex_material` entries telling the client how many faces there are
    /// that use a particular material. This field only exists if bit 12 of `flags` is set.
    pub vertex_material_group_count: Option<u32>,

    /// The first element of the tuple is the number of vertices that use the same
    /// material. Vertex materials, like faces, are sorted by material index so
    /// that vertices that use the same material are together.
    ///
    /// The second element of the tuple is the index of the material that the
    /// vertices use, according to the [MaterialPalette] fragment that this fragment
    /// references. This field only exists if bit 12 of `flags` is set.
    ///
    /// The rest are _Unknown_
    ///
    /// There are 'vertex_material_group_count` of these.
    pub vertex_material_groups: Option<Vec<(u16, u16)>>,

    /// its purpose is unknown. This field only exists if bit 13 of Flags is 1.
    pub params2: Option<(u32, u32, u32)>,

    /// its purpose is unknown. This field only exists if bit 14 of Flags is 1.
    pub params3: Option<(f32, f32, f32, f32, f32, f32)>,
}

impl FragmentParser for DmSpriteDef {
    type T = Self;

    const TYPE_ID: u32 = 0x2c;
    const TYPE_NAME: &'static str = "DmSpriteDef";

    fn parse(input: &[u8]) -> WResult<'_, DmSpriteDef> {
        let (
            i,
            (
                name_reference,
                flags,
                vertex_count,
                texture_coordinate_count,
                normal_count,
                color_count,
                face_count,
                meshop_count,
                fragment1,
                skin_assignment_group_count,
                material_list_ref,
                fragment3,
                center,
                params1,
            ),
        ) = (
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
            FragmentRef::parse,
            le_u32,
            (le_f32, le_f32, le_f32),
            (le_f32, le_f32, le_f32),
        )
            .parse(input)?;

        let (
            i,
            (
                vertices,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                faces,
                meshops,
                skin_assignment_groups,
            ),
        ) = (
            count((le_f32, le_f32, le_f32), vertex_count as usize),
            count((le_f32, le_f32), texture_coordinate_count as usize),
            count((le_f32, le_f32, le_f32), normal_count as usize),
            count(le_u32, color_count as usize),
            count(DmSpriteDefFaceEntry::parse, face_count as usize),
            count(DmSpriteDefMeshopEntry::parse, meshop_count as usize),
            count((le_u16, le_u16), skin_assignment_group_count as usize),
        )
            .parse(i)?;

        let (i, size8) = if flags & 0x200 == 0x200 {
            // Bit 9 is set
            le_u32(i).map(|(i, size8)| (i, Some(size8)))?
        } else {
            (i, None)
        };

        let (i, data8) = if flags & 0x200 == 0x200 {
            // Bit 9 is set
            count(le_u32, size8.unwrap() as usize)
                .parse(i)
                .map(|(i, data8)| (i, Some(data8)))?
        } else {
            (i, None)
        };

        let (i, face_material_group_count) = if flags & 0x800 == 0x800 {
            // Bit 11 set
            le_u32(i).map(|(i, face_material_group_count)| (i, Some(face_material_group_count)))?
        } else {
            (i, None)
        };

        let (i, face_material_groups) = if flags & 0x800 == 0x800 {
            // Bit 11 set
            count(
                (le_u16, le_u16),
                face_material_group_count.unwrap() as usize,
            )
            .parse(i)
            .map(|(i, face_material_groups)| (i, Some(face_material_groups)))?
        } else {
            (i, None)
        };

        let (i, vertex_material_group_count) = if flags & 0x1000 == 0x1000 {
            // Bit 12 set
            le_u32(i)
                .map(|(i, vertex_material_group_count)| (i, Some(vertex_material_group_count)))?
        } else {
            (i, None)
        };

        let (i, vertex_material_groups) = if flags & 0x1000 == 0x1000 {
            // Bit 12 set
            count(
                (le_u16, le_u16),
                vertex_material_group_count.unwrap() as usize,
            )
            .parse(i)
            .map(|(i, vertex_material_groups)| (i, Some(vertex_material_groups)))?
        } else {
            (i, None)
        };

        let (i, params2) = if flags & 0x2000 == 0x2000 {
            // Bit 13 set
            (le_u32, le_u32, le_u32)
                .parse(i)
                .map(|(i, params2)| (i, Some(params2)))?
        } else {
            (i, None)
        };

        let (i, params3) = if flags & 0x4000 == 0x4000 {
            // Bit 14 set
            (le_f32, le_f32, le_f32, le_f32, le_f32, le_f32)
                .parse(i)
                .map(|(i, params3)| (i, Some(params3)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            DmSpriteDef {
                name_reference,
                flags,
                vertex_count,
                texture_coordinate_count,
                normal_count,
                color_count,
                face_count,
                meshop_count,
                fragment1,
                skin_assignment_group_count,
                material_list_ref,
                fragment3,
                center,
                params1,
                vertices,
                texture_coordinates,
                vertex_normals,
                vertex_colors,
                faces,
                meshops,
                skin_assignment_groups,
                size8,
                data8,
                face_material_group_count,
                face_material_groups,
                vertex_material_group_count,
                vertex_material_groups,
                params2,
                params3,
            },
        ))
    }
}

impl Fragment for DmSpriteDef {
    fn into_bytes(&self) -> Vec<u8> {
        let vertices = self
            .vertices
            .iter()
            .flat_map(|v| vec![v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()])
            .flatten()
            .collect::<Vec<_>>();
        let texture_coordinates = self
            .texture_coordinates
            .iter()
            .flat_map(|v| vec![v.0.to_le_bytes(), v.1.to_le_bytes()])
            .flatten()
            .collect::<Vec<_>>();
        let vertex_normals = self
            .vertex_normals
            .iter()
            .flat_map(|v| vec![v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()])
            .flatten()
            .collect::<Vec<_>>();
        let faces = self
            .faces
            .iter()
            .flat_map(|p| p.into_bytes())
            .collect::<Vec<_>>();
        let meshops = self
            .meshops
            .iter()
            .flat_map(|d| d.into_bytes())
            .collect::<Vec<_>>();

        let data8 = self.data8.as_ref().map_or(vec![], |i| {
            i.iter().flat_map(|d| d.to_le_bytes()).collect::<Vec<_>>()
        });

        let skin_assignment_groups = self
            .skin_assignment_groups
            .iter()
            .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes()].concat())
            .collect::<Vec<_>>();

        let face_material_groups = self.face_material_groups.as_ref().map_or(vec![], |i| {
            i.iter()
                .flat_map(|v| [&v.0.to_le_bytes()[..], &v.1.to_le_bytes()[..]].concat())
                .collect::<Vec<_>>()
        });

        let vertex_material_groups = self.vertex_material_groups.as_ref().map_or(vec![], |i| {
            i.iter()
                .flat_map(|v| [&v.0.to_le_bytes()[..], &v.1.to_le_bytes()[..]].concat())
                .collect::<Vec<_>>()
        });

        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.vertex_count.to_le_bytes()[..],
            &self.texture_coordinate_count.to_le_bytes()[..],
            &self.normal_count.to_le_bytes()[..],
            &self.color_count.to_le_bytes()[..],
            &self.face_count.to_le_bytes()[..],
            &self.meshop_count.to_le_bytes()[..],
            &self.fragment1.to_le_bytes()[..],
            &self.skin_assignment_group_count.to_le_bytes()[..],
            &self.material_list_ref.into_bytes()[..],
            &self.fragment3.to_le_bytes()[..],
            &self.center.0.to_le_bytes()[..],
            &self.center.1.to_le_bytes()[..],
            &self.center.2.to_le_bytes()[..],
            &self.params1.0.to_le_bytes()[..],
            &self.params1.1.to_le_bytes()[..],
            &self.params1.2.to_le_bytes()[..],
            &vertices[..],
            &texture_coordinates[..],
            &vertex_normals[..],
            &self
                .vertex_colors
                .iter()
                .flat_map(|d| d.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &faces[..],
            &meshops[..],
            &skin_assignment_groups[..],
            &self.size8.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &data8[..],
            &self
                .face_material_group_count
                .map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &face_material_groups[..],
            &self
                .vertex_material_group_count
                .map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &vertex_material_groups[..],
            &self.params2.map_or(vec![], |p| {
                [p.0.to_le_bytes(), p.1.to_le_bytes(), p.2.to_le_bytes()].concat()
            })[..],
            &self.params3.map_or(vec![], |p| {
                [
                    p.0.to_le_bytes(),
                    p.1.to_le_bytes(),
                    p.2.to_le_bytes(),
                    p.3.to_le_bytes(),
                    p.4.to_le_bytes(),
                    p.5.to_le_bytes(),
                ]
                .concat()
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
/// Represents a polygon within a [DmSpriteDef].
pub struct DmSpriteDefFaceEntry {
    /// This is usually 0 except the first face which has a unique value each time (probably not flags)
    pub flags: u16,

    /// _Unknown_ - Usually contains zeros.
    /// In gequip, this is filled with vertex indexes (except the first face, which has big numbers) and vertex_indexes is not.  4th element is always 75
    pub data: (u16, u16, u16, u16),

    /// An index for each of the polygon's vertex coordinates (idx1, idx2, idx3).
    /// This must be incorrect, this is usually `0,1,0` or `0,0,0` or `0,2,0` or something similar - not vertex indexes
    pub vertex_indexes: (u16, u16, u16),
}

impl DmSpriteDefFaceEntry {
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

impl DmSpriteDefFaceEntry {
    fn parse(input: &[u8]) -> WResult<'_, DmSpriteDefFaceEntry> {
        let (remaining, (flags, data, vertex_indexes)) = (
            le_u16,
            (le_u16, le_u16, le_u16, le_u16),
            (le_u16, le_u16, le_u16),
        )
            .parse(input)?;
        Ok((
            remaining,
            DmSpriteDefFaceEntry {
                flags,
                data,
                vertex_indexes,
            },
        ))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct DmSpriteDefMeshopEntry {
    /// _Unknown_ - It seems to control whether VertexIndex1, VertexIndex2, and Offset exist.
    /// It can only contain values in the range 1 to 4. It looks like the Data9 entries are broken
    /// up into blocks, where each block is terminated by an entry where Data9Type is 4.
    pub type_field: u32,

    /// This seems to reference one of the vertex entries. This field only exists if `_type`
    /// contains a value in the range 1 to 3.
    pub vertex_index: Option<u32>,

    /// _Unknown_
    /// If `_type` contains 4 then this field exists instead of `vertex_index`. meshops entries seem
    /// to be sorted by this value
    pub offset: Option<f32>,

    /// _Unknown_ - Seems to only contain values in the range 0 to 2.
    pub param1: u16,

    /// _Unknown_
    pub param2: u16,
}

impl DmSpriteDefMeshopEntry {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.type_field.to_le_bytes()[..],
            &self
                .vertex_index
                .map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &self.offset.map_or(vec![], |i| i.to_le_bytes().to_vec())[..],
            &self.param1.to_le_bytes()[..],
            &self.param2.to_le_bytes()[..],
        ]
        .concat()
    }
}

impl DmSpriteDefMeshopEntry {
    fn parse(input: &[u8]) -> WResult<'_, DmSpriteDefMeshopEntry> {
        let (remaining, type_field) = le_u32(input)?;

        let (remaining, offset) = if type_field == 4 {
            le_f32(remaining).map(|(i, offset)| (i, Some(offset)))?
        } else {
            (remaining, None)
        };

        let (remaining, vertex_index) = if type_field != 4 {
            le_u32(remaining).map(|(i, vertex_index)| (i, Some(vertex_index)))?
        } else {
            (remaining, None)
        };

        let (remaining, (param1, param2)) = (le_u16, le_u16).parse(remaining)?;

        Ok((
            remaining,
            DmSpriteDefMeshopEntry {
                type_field,
                vertex_index,
                offset,
                param1,
                param2,
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
        let frag = DmSpriteDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-44));
        assert_eq!(frag.flags, 0x1802);
        assert_eq!(frag.vertex_count, 29);
        assert_eq!(frag.texture_coordinate_count, 29);
        assert_eq!(frag.normal_count, 29);
        assert_eq!(frag.color_count, 0);
        assert_eq!(frag.face_count, 12);
        assert_eq!(frag.meshop_count, 64);
        assert_eq!(frag.skin_assignment_group_count, 1);
        assert_eq!(frag.material_list_ref, FragmentRef::new(5));
        assert_eq!(frag.center, (0.0, 0.0, 0.0));
        assert_eq!(frag.params1, (0.0, 0.0, 12.247571));
        assert_eq!(frag.vertices.len(), frag.vertex_count as usize);
        assert_eq!(frag.vertices[0], (-5.070387, -5.073263, 4.9999995));
        assert_eq!(
            frag.texture_coordinates.len(),
            frag.texture_coordinate_count as usize
        );
        assert_eq!(frag.texture_coordinates[0], (0.99999994, 0.99999976)); // FIXME: This does not look like a valid UV coordinate.  It should be between 0 and 1.
        assert_eq!(frag.vertex_normals.len(), frag.normal_count as usize);
        assert_eq!(frag.vertex_normals[0], (0.0, -1.0, 1.3435886e-7));
        assert_eq!(frag.vertex_colors.len(), frag.color_count as usize);
        assert_eq!(frag.faces.len(), frag.face_count as usize);
        assert_eq!(frag.faces[1].flags, 0b1001011);
        assert_eq!(frag.faces[0].data, (0, 0, 0, 0));
        assert_eq!(frag.faces[0].vertex_indexes, (3, 2, 0));
        assert_eq!(frag.meshops.len(), frag.meshop_count as usize);
        assert_eq!(frag.meshops[0].param1, 0);
        assert_eq!(frag.meshops[0].param2, 0);
        assert_eq!(frag.meshops[0].type_field, 2);
        assert_eq!(
            frag.skin_assignment_groups.len(),
            frag.skin_assignment_group_count as usize
        );
        assert_eq!(frag.size8, None);
        assert_eq!(frag.data8, None);
        assert_eq!(frag.face_material_group_count, Some(1));
        assert_eq!(frag.face_material_groups, Some(vec![(12, 0)]));
        assert_eq!(frag.vertex_material_group_count, Some(1));
        assert_eq!(frag.vertex_material_groups, Some(vec![(29, 0)]));
    }

    #[test]
    fn it_parses_with_bit14() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip_beta/0567-0x2c.frag")[..];
        let frag = DmSpriteDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-6091));
        assert_eq!(frag.flags, 0x5803);
        assert_eq!(frag.vertex_count, 56);
        assert_eq!(frag.texture_coordinate_count, 56);
        assert_eq!(frag.normal_count, 56);
        assert_eq!(frag.color_count, 0);
        assert_eq!(frag.face_count, 28);
        assert_eq!(frag.meshop_count, 0);
        assert_eq!(frag.skin_assignment_group_count, 0);
        assert_eq!(frag.material_list_ref, FragmentRef::new(567));
        assert_eq!(frag.center, (0.0, 0.0, 2.2031567));
        assert_eq!(frag.params1, (-0.002979297, -0.88816565, 3.0285614));
        assert_eq!(frag.vertices.len(), frag.vertex_count as usize);
        assert_eq!(
            frag.texture_coordinates.len(),
            frag.texture_coordinate_count as usize
        );
        assert_eq!(frag.vertex_normals.len(), frag.normal_count as usize);
        assert_eq!(frag.vertex_colors.len(), frag.color_count as usize);
        assert_eq!(frag.faces.len(), frag.face_count as usize);
        assert_eq!(frag.meshops.len(), frag.meshop_count as usize);
        assert_eq!(
            frag.skin_assignment_groups.len(),
            frag.skin_assignment_group_count as usize
        );
        assert_eq!(frag.size8, None);
        assert_eq!(frag.data8, None);
        assert_eq!(frag.face_material_group_count, Some(2));
        assert_eq!(frag.vertex_material_group_count, Some(2));
        assert_eq!(frag.params3.unwrap().0, -2.871873);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0005-0x2c.frag")[..];
        let frag = DmSpriteDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
