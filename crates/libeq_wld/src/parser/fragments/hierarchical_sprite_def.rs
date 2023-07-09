use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// This fragment describes a skeleton for an entire animated model, and is used for mob
/// models. The overall skeleton is contained in a 0x10 [HierarchicalSpriteDef] and
/// is structured as a hierarchical tree. For example, a pelvis piece might connect to chest,
/// left thigh, and right thigh pieces. The chest piece might connect to left bicep, right
/// bicep, and neck pieces. The left bicep piece might connect to a left forearm piece.
/// The left forearm piece might connect to a left hand piece. The idea is to start at the
/// base “stem” piece in the skeleton and recursively walk the tree to each successive piece.
///
/// For each piece there is a 0x13 [Track], which
/// references one 0x12 [MobSkeletonPieceTrackFragment]. Each 0x12 fragment defines
/// how that piece is rotated and/or shifted relative to its parent piece.
///
/// **Type ID:** 0x10
pub struct HierarchicalSpriteDef {
    pub name_reference: StringReference,

    /// Most flags are _unknown_.
    /// * bit 0 - If set then `center_offset` exists.
    /// * bit 1 - If set then `bounding_radius` exists.
    /// * bit 9 - If set then `size2`, `fragment3`, and `data3` exist.
    pub flags: HierarchicalSpriteDefFlags,

    /// The number of track reference entries
    pub num_dags: u32,

    /// Optionally points to a 0x18 [PolygonAnimationReferenceFragment]?
    /// TODO: This still needs investigation
    pub collision_volume_reference: u32,

    pub center_offset: Option<(u32, u32, u32)>,

    pub bounding_radius: Option<f32>,

    /// There are `num_dags` entries.
    pub dags: Vec<Dag>,

    /// The number of fragment3 and data3 entries there are.
    pub num_attached_skins: Option<u32>,

    /// There are `num_attched_skins` of these. This field only exists if the proper bit in the `flags`
    /// field is set. These entries generally point to 0x2D [MeshReferenceFragment]s and
    /// outline all of the meshes in the animated model. For example, there might be a mesh
    /// for a model’s body and another one for the head.
    pub dm_sprites: Option<Vec<u32>>,

    /// _Unknown_ - There are size2 of these.
    pub link_skin_updates_to_dag_index: Option<Vec<u32>>,
}

impl FragmentParser for HierarchicalSpriteDef {
    type T = Self;

    const TYPE_ID: u32 = 0x10;
    const TYPE_NAME: &'static str = "HierarchicalSpriteDef";

    fn parse(input: &[u8]) -> WResult<HierarchicalSpriteDef> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = HierarchicalSpriteDefFlags::parse(i)?;
        let (i, num_dags) = le_u32(i)?;
        let (i, collision_volume_reference) = le_u32(i)?;

        let (i, center_offset) = if flags.has_center_offset() {
            tuple((le_u32, le_u32, le_u32))(i).map(|(i, p1)| (i, Some(p1)))?
        } else {
            (i, None)
        };

        let (i, bounding_radius) = if flags.has_bounding_radius() {
            le_f32(i).map(|(i, p2)| (i, Some(p2)))?
        } else {
            (i, None)
        };

        let (i, dags) = count(Dag::parse, num_dags as usize)(i)?;

        let (i, num_attached_skins) = if flags.has_unknown_flag() {
            le_u32(i).map(|(i, size2)| (i, Some(size2)))?
        } else {
            (i, None)
        };

        let (remaining, (dm_sprites, link_skin_updates_to_dag_index)) = if flags.has_unknown_flag()
        {
            let size = num_attached_skins.unwrap_or(0) as usize;
            tuple((count(le_u32, size), count(le_u32, size)))(i)
                .map(|(i, (f3, d3))| (i, (Some(f3), Some(d3))))?
        } else {
            (i, (None, None))
        };

        Ok((
            remaining,
            HierarchicalSpriteDef {
                name_reference,
                flags,
                num_dags,
                collision_volume_reference,
                center_offset,
                bounding_radius,
                dags,
                num_attached_skins,
                dm_sprites,
                link_skin_updates_to_dag_index,
            },
        ))
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Entries in the map's [HierarchicalSpriteDef]
pub struct Dag {
    /// This seems to refer to the name of either this or another 0x10 fragment.
    /// It seems that at least one name reference points to the name of this fragment.
    pub name_reference: i32,

    /// _Unknown_ - Usually 0x0
    pub flags: u32,

    /// Reference to a 0x13 Mob Skeleton Piece Track Reference fragment.
    ///
    /// Important: animated models generally only reference a basic set of fragments
    /// necessary to render the model but not animate it. There will generally be
    /// other sets of 0x13 fragments where each set corresponds to a different
    /// animation of the model. Software reading .WLD files must use the name of
    /// the first 0x13 fragment referenced by the 0x10 Skeleton Track Set to discover
    /// any other animation sets. The first fragment of any alternate animation set
    /// will have the same name as the first 0x13 fragment, with an additional prefix.
    /// All other 0x13 fragments in that same set will likewise correspond to their
    /// counterparts in the basic animation set. Different animation sets will have
    /// different prefixes (e.g. “C01” for one combat animation, “C02” for another
    /// combat animation, etc.). All alternate animation sets for a particular model
    /// generally immediately follow the 0x10 Skeleton Track Set fragment (with the
    /// 0x11 Skeleton Track Set Reference immediately following those). I don’t know
    /// if this is a necessary arrangement.
    pub track_reference: u32,

    /// Sometimes refers to a 0x2D Mesh Reference fragment.
    pub mesh_or_sprite_reference: u32,

    /// The number of data entries
    pub num_sub_dags: u32,

    /// Each of these contains the index of the next piece in the skeleton tree. A
    /// Skeleton Track Set is a hierarchical tree of pieces in the skeleton. It
    /// generally starts with a central “stem” and branches out to a skeleton’s
    /// extremities. For instance, the first entry might be the stem; that entry
    /// might point to the pelvis entry; the pelvis entry might point to the left thigh,
    /// right thigh, and chest entries; and those entries would each point to other parts
    /// of the skeleton. The exact topography of the tree depends upon the overall
    /// structure of the skeleton. The proper way to use a Skeleton Track Set fragment
    /// is to start with the first entry and recursively walk the tree by following each
    /// entry’s Entry1Data field to other connected pieces.
    ///
    /// It’s also worth noting that, although an entry might reference a 0x13 Mob Skeleton
    /// Piece Track Reference fragment in its EntityFragment1 field, that does not mean it
    /// will be valid for rendering (see the 0x12 Mob Skeleton Piece Track fragment for more
    /// information). Many model skeletons apparently contain extraneous pieces that have an
    /// unknown purpose, though I suspect that they are for determining attachment points
    /// for weapons and shields and are otherwise not meant to be rendered. These pieces are
    /// generally not referenced by the 0x36 Mesh fragments that the skeleton indirectly
    /// references (via 0x2D Mesh Reference fragments).
    pub sub_dags: Vec<u32>,
}

impl Dag {
    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = le_i32(input)?;
        let (i, flags) = le_u32(i)?;
        let (i, track_reference) = le_u32(i)?;
        let (i, mesh_or_sprite_reference) = le_u32(i)?;
        let (i, num_sub_dags) = le_u32(i)?;
        let (remaining, sub_dags) = count(le_u32, num_sub_dags as usize)(i)?;

        Ok((
            remaining,
            Self {
                name_reference,
                flags,
                track_reference,
                mesh_or_sprite_reference,
                num_sub_dags,
                sub_dags,
            },
        ))
    }

    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_le_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.track_reference.to_le_bytes()[..],
            &self.mesh_or_sprite_reference.to_le_bytes()[..],
            &self.num_sub_dags.to_le_bytes()[..],
            &self
                .sub_dags
                .iter()
                .flat_map(|d| d.to_le_bytes())
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }
}

impl Fragment for HierarchicalSpriteDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.num_dags.to_le_bytes()[..],
            &self.collision_volume_reference.to_le_bytes()[..],
            &self.center_offset.map_or(vec![], |p| {
                [p.0.to_le_bytes(), p.1.to_le_bytes(), p.2.to_le_bytes()].concat()
            })[..],
            &self
                .bounding_radius
                .map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self
                .dags
                .iter()
                .flat_map(|e| e.into_bytes())
                .collect::<Vec<_>>()[..],
            &self
                .num_attached_skins
                .map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self
                .dm_sprites
                .as_ref()
                .map_or(vec![], |f| f.iter().flat_map(|x| x.to_le_bytes()).collect())[..],
            &self
                .link_skin_updates_to_dag_index
                .as_ref()
                .map_or(vec![], |d| d.iter().flat_map(|x| x.to_le_bytes()).collect())[..],
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
pub struct HierarchicalSpriteDefFlags(u32);

impl HierarchicalSpriteDefFlags {
    const HAS_CENTER_OFFSET: u32 = 0x01;
    const HAS_BOUNDING_RADIUS: u32 = 0x02;
    const UNKNOWN_FLAG: u32 = 0x200;

    fn parse(input: &[u8]) -> WResult<Self> {
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

    pub fn has_unknown_flag(&self) -> bool {
        self.0 & Self::UNKNOWN_FLAG == Self::UNKNOWN_FLAG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0013-0x10.frag")[..];
        let frag = HierarchicalSpriteDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-211));
        assert_eq!(frag.num_dags, 3);
        assert_eq!(frag.collision_volume_reference, 0);
        assert_eq!(frag.center_offset, None);
        assert_eq!(frag.bounding_radius, Some(0.81542796));
        assert_eq!(frag.dags.len(), 3);
        assert_eq!(frag.dags[0].name_reference, -168);
        assert_eq!(frag.dags[0].flags, 0);
        assert_eq!(frag.dags[0].track_reference, 8);
        assert_eq!(frag.dags[0].mesh_or_sprite_reference, 0);
        assert_eq!(frag.dags[0].num_sub_dags, 1);
        assert_eq!(frag.dags[0].sub_dags.len(), 1);
        assert_eq!(frag.dags[0].sub_dags[0], 1);
        assert_eq!(frag.num_attached_skins, Some(1));
        assert_eq!(frag.dm_sprites.unwrap(), vec![13]);
        assert_eq!(frag.link_skin_updates_to_dag_index.unwrap(), vec![2]);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0013-0x10.frag")[..];
        let frag = HierarchicalSpriteDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
