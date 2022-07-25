use std::any::Any;

use super::{BspRegionFragment, Fragment, FragmentParser, FragmentRef, StringReference};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
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

impl FragmentParser for BspTreeFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x21;
    const TYPE_NAME: &'static str = "BspTree";

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
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .entries
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
            FragmentRef::parse,
            tuple((FragmentRef::parse, FragmentRef::parse)),
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

    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.normal.0.to_le_bytes()[..],
            &self.normal.1.to_le_bytes()[..],
            &self.normal.2.to_le_bytes()[..],
            &self.split_distance.to_le_bytes()[..],
            &self.region.into_bytes()[..],
            &self.nodes.0.into_bytes()[..],
            &self.nodes.1.into_bytes()[..],
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
        let frag = BspTreeFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0x0));
        assert_eq!(frag.size1, 5809);
        assert_eq!(frag.entries.len(), 5809);
        assert_eq!(frag.entries[0].normal, (-1.0f32, 0.0f32, 0.0f32));
        assert_eq!(frag.entries[0].split_distance, -187.8942f32);
        assert_eq!(frag.entries[0].region, FragmentRef::new(0));
        assert_eq!(
            frag.entries[0].nodes,
            (FragmentRef::new(2), FragmentRef::new(2507))
        );
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1730-0x21.frag")[..];
        let frag = BspTreeFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
