use std::any::Any;

use super::{
    fragment_ref, BspRegionFragment, Fragment, FragmentRef, FragmentType, StringReference,
};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A map's BSP Tree.
///
/// **Type ID:** 0x21
pub struct BspTreeFragment {
    pub name_reference: StringReference,

    /// The number of [BspTreeFragmentEntry]s in this tree.
    pub size1: u32,

    /// The [BspTreeFragmentEntry]s
    pub entries: Vec<BspTreeFragmentEntry>,
}

impl FragmentType for BspTreeFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x21;

    fn parse(input: &[u8]) -> IResult<&[u8], BspTreeFragment> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, size1) = le_u32(i)?;
        let (remaining, entries) = count(BspTreeFragmentEntry::parse, size1 as usize)(i)?;

        Ok((
            remaining,
            BspTreeFragment {
                name_reference,
                size1,
                entries,
            },
        ))
    }
}

impl Fragment for BspTreeFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .entries
                .iter()
                .flat_map(|e| e.serialize())
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

    /// If this is not a leaf node these are references to [BspTreeFragmentEntry] on either side of the
    /// splitting plane.
    pub nodes: (
        FragmentRef<BspTreeFragmentEntry>,
        FragmentRef<BspTreeFragmentEntry>,
    ),
}

impl BspTreeFragmentEntry {
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

    fn serialize(&self) -> Vec<u8> {
        [
            &self.normal.0.to_le_bytes()[..],
            &self.normal.1.to_le_bytes()[..],
            &self.normal.2.to_le_bytes()[..],
            &self.split_distance.to_le_bytes()[..],
            &self.region.serialize()[..],
            &self.nodes.0.serialize()[..],
            &self.nodes.1.serialize()[..],
        ]
        .concat()
    }
}
