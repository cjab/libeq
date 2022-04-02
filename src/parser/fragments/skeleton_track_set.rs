use std::any::Any;

use super::{Fragment, FragmentType, StringReference};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// This fragment describes a skeleton for an entire animated model, and is used for mob
/// models. The overall skeleton is contained in a 0x10 [SkeletonTrackSetFragment] and
/// is structured as a hierarchical tree. For example, a pelvis piece might connect to chest,
/// left thigh, and right thigh pieces. The chest piece might connect to left bicep, right
/// bicep, and neck pieces. The left bicep piece might connect to a left forearm piece.
/// The left forearm piece might connect to a left hand piece. The idea is to start at the
/// base “stem” piece in the skeleton and recursively walk the tree to each successive piece.
///
/// For each piece there is a 0x13 [MobSkeletonPieceTrackReferenceFragment], which
/// references one 0x12 [MobSkeletonPieceTrackFragment]. Each 0x12 fragment defines
/// how that piece is rotated and/or shifted relative to its parent piece.
///
/// **Type ID:** 0x10
pub struct SkeletonTrackSetFragment {
    pub name_reference: StringReference,

    /// Most flags are _unknown_.
    /// * bit 0 - If set then `unknown_params1` exists.
    /// * bit 1 - If set then `unknown_params2` exists.
    /// * bit 9 - If set then `size2`, `fragment3`, and `data3` exist.
    pub flags: u32,

    /// The number of track reference entries
    pub entry_count: u32,

    /// Optionally points to a 0x18 [PolygonAnimationReferenceFragment]?
    pub fragment: u32,

    /// _Unknown_
    pub unknown_params1: Option<(u32, u32, u32)>,

    /// _Unknown_
    pub unknown_params2: Option<f32>,

    /// There are `entry_count` entries.
    pub entries: Vec<SkeletonTrackSetFragmentEntry>,

    /// The number of fragment3 and data3 entries there are.
    pub size2: Option<u32>,

    /// There are `size2` of these. This field only exists if the proper bit in the `flags`
    /// field is set. These entries generally point to 0x2D [MeshReferenceFragment]s and
    /// outline all of the meshes in the animated model. For example, there might be a mesh
    /// for a model’s body and another one for the head.
    pub fragment3: Option<Vec<u32>>,

    /// _Unknown_ - There are size2 of these.
    pub data3: Option<Vec<u32>>,
}

impl FragmentType for SkeletonTrackSetFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x10;

    fn parse(input: &[u8]) -> IResult<&[u8], SkeletonTrackSetFragment> {
        let (i, (name_reference, flags, entry_count, fragment)) =
            tuple((StringReference::parse, le_u32, le_u32, le_u32))(input)?;

        let (i, unknown_params1) = if flags & 0x01 == 0x01 {
            tuple((le_u32, le_u32, le_u32))(i).map(|(i, p1)| (i, Some(p1)))?
        } else {
            (i, None)
        };

        let (i, unknown_params2) = if flags & 0x02 == 0x02 {
            le_f32(i).map(|(i, p2)| (i, Some(p2)))?
        } else {
            (i, None)
        };

        let (i, entries) = count(SkeletonTrackSetFragmentEntry::parse, entry_count as usize)(i)?;

        let (i, size2) = if flags & 0x200 == 0x200 {
            le_u32(i).map(|(i, size2)| (i, Some(size2)))?
        } else {
            (i, None)
        };

        let (remaining, (fragment3, data3)) = if flags & 0x200 == 0x200 {
            let size = size2.unwrap_or(0) as usize;
            tuple((count(le_u32, size), count(le_u32, size)))(i)
                .map(|(i, (f3, d3))| (i, (Some(f3), Some(d3))))?
        } else {
            (i, (None, None))
        };

        Ok((
            remaining,
            SkeletonTrackSetFragment {
                name_reference,
                flags,
                entry_count,
                fragment,
                unknown_params1,
                unknown_params2,
                entries,
                size2,
                fragment3,
                data3,
            },
        ))
    }
}

#[derive(Debug)]
/// Entries in the map's [SkeletonTrackSetFragment]
pub struct SkeletonTrackSetFragmentEntry {
    /// This seems to refer to the name of either this or another 0x10 fragment.
    /// It seems that at least one name reference points to the name of this fragment.
    pub name_reference: u32,

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
    pub fragment1: u32,

    /// Sometimes refers to a 0x2D Mesh Reference fragment.
    pub fragment2: u32,

    /// The number of data entries
    pub data_entry_count: u32,

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
    pub data_entries: Vec<u32>,
}

impl SkeletonTrackSetFragmentEntry {
    fn parse(input: &[u8]) -> IResult<&[u8], SkeletonTrackSetFragmentEntry> {
        let (i, (name_reference, flags, fragment1, fragment2, data_entry_count)) =
            tuple((le_u32, le_u32, le_u32, le_u32, le_u32))(input)?;

        let (remaining, data_entries) = count(le_u32, data_entry_count as usize)(i)?;

        Ok((
            remaining,
            SkeletonTrackSetFragmentEntry {
                name_reference,
                flags,
                fragment1,
                fragment2,
                data_entry_count,
                data_entries,
            },
        ))
    }

    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.to_le_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.fragment1.to_le_bytes()[..],
            &self.fragment2.to_le_bytes()[..],
            &self.data_entry_count.to_le_bytes()[..],
            &self
                .data_entries
                .iter()
                .flat_map(|d| d.to_le_bytes())
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }
}

impl Fragment for SkeletonTrackSetFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.entry_count.to_le_bytes()[..],
            &self.fragment.to_le_bytes()[..],
            &self.unknown_params1.map_or(vec![], |p| {
                [p.0.to_le_bytes(), p.1.to_le_bytes(), p.2.to_le_bytes()].concat()
            })[..],
            &self
                .unknown_params2
                .map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self
                .entries
                .iter()
                .flat_map(|e| e.serialize())
                .collect::<Vec<_>>()[..],
            &self.size2.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self
                .fragment3
                .as_ref()
                .map_or(vec![], |f| f.iter().flat_map(|x| x.to_le_bytes()).collect())[..],
            &self
                .data3
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
}
