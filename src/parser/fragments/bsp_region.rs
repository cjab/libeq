use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, MeshFragment, StringHash};

use nom::combinator::map;
use nom::multi::count;
use nom::number::complete::{le_u16, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A region within a map's BSP Tree.
///
/// **Type ID:** 0x22
pub struct BspRegionFragment {
    /// Most flags are _unknown_. Usually contains 0x181 for regions that contain polygons and 0x81
    /// for regions that are empty.
    /// * bit 5 - If set then `pvs` contains u32 entries.
    /// * bit 7 - If set then `pvs` contains u8 entries (more common).
    pub flags: u32,

    /// _Unknown_ - Some sort of fragment reference. Usually nothing is referenced.
    pub fragment1: FragmentRef<i32>,

    /// The number of bytes in `data1`
    pub size1: u32,

    /// The number of bytes in `data2`
    pub size2: u32,

    /// _Unknown_ - Usually 0
    pub params1: u32,

    /// The number of `data3` entries. Usually 0.
    pub size3: u32,

    /// The number of `data4` entries. Usually 0.
    pub size4: u32,

    /// _Unknown_ - Usually 0.
    pub params2: u32,

    /// The number of `data5` entries. Usually 1.
    pub size5: u32,

    /// The number of `pvs` entries. Usually 1.
    pub pvs_count: u32,

    /// According to the ZoneConverter source there are 12 * `size1` bytes here. Their format is
    /// _unknown_ for lack of sample data to figure it out.
    pub data1: Vec<u8>,

    /// According to the ZoneConverter source there are 8 * `size2` bytes here. Their format is
    /// _unknown_ for lack of sample data to figure it out.
    pub data2: Vec<u8>,

    /// _Unknown_ data entries
    pub data3: Vec<BspRegionFragmentData3Entry>,

    /// _Unknown_ data entries
    pub data4: Vec<BspRegionFragmentData4Entry>,

    /// _Unknown_ data entries
    pub data5: Vec<BspRegionFragmentData5Entry>,

    /// A potentially visible set (PVS) of regions
    pub pvs: Vec<BspRegionFragmentPVS>,

    /// The number of bytes in the `name7` field.
    pub size7: u32,

    /// _Unknown_ - An encoded string.
    pub name7: Vec<u8>,

    /// _Unknown_ - Usually references nothing.
    pub fragment2: FragmentRef<i32>,

    /// If there are any polygons in this region then this reference points to a [MeshFragment]
    /// that contains only those polygons. That [MeshFragment] must contain all geometry information
    /// contained within the volume that this region represents and nothing that lies outside of
    /// that volume.
    pub mesh_reference: Option<FragmentRef<MeshFragment>>,
}

impl FragmentType for BspRegionFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x22;

    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragment> {
        let (i, (flags, fragment1, size1, size2, params1, size3, size4, params2, size5, pvs_count)) =
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
        let (i, (data1, data2, data3, data4, data5, pvs, size7)) = tuple((
            count(le_u8, size1 as usize),
            count(le_u8, size2 as usize),
            count(BspRegionFragmentData3Entry::parse, size3 as usize),
            count(BspRegionFragmentData4Entry::parse, size4 as usize),
            count(BspRegionFragmentData5Entry::parse, size5 as usize),
            count(BspRegionFragmentPVS::parse, pvs_count as usize),
            le_u32,
        ))(i)?;
        let (i, (name7, fragment2)) = tuple((count(le_u8, 12), fragment_ref))(i)?;

        let (remaining, mesh_reference) = if (flags & 0x100) == 0x100 {
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
                pvs_count,
                data1,
                data2,
                data3,
                data4,
                data5,
                pvs,
                size7,
                name7,
                fragment2,
                mesh_reference,
            },
        ))
    }
}

impl Fragment for BspRegionFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.flags.to_le_bytes()[..],
            &self.fragment1.serialize()[..],
            &self.size1.to_le_bytes()[..],
            &self.size2.to_le_bytes()[..],
            &self.params1.to_le_bytes()[..],
            &self.size3.to_le_bytes()[..],
            &self.size4.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self.size5.to_le_bytes()[..],
            &self.pvs_count.to_le_bytes()[..],
            &self.data2[..],
            &self
                .data3
                .iter()
                .flat_map(|d| d.serialize())
                .collect::<Vec<_>>()[..],
            &self
                .data4
                .iter()
                .flat_map(|d| d.serialize())
                .collect::<Vec<_>>()[..],
            &self
                .data5
                .iter()
                .flat_map(|d| d.serialize())
                .collect::<Vec<_>>()[..],
            &self
                .pvs
                .iter()
                .flat_map(|p| p.serialize())
                .collect::<Vec<_>>()[..],
            &self.size7.to_le_bytes()[..],
            &self.name7,
            &self.fragment2.serialize()[..],
            &self
                .mesh_reference
                .as_ref()
                .map_or(vec![], |m| m.serialize())[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self, string_hash: &StringHash) -> String {
        String::new()
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

impl BspRegionFragmentData3Entry {
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

    fn serialize(&self) -> Vec<u8> {
        [
            &self.flags.to_le_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .data1
                .iter()
                .flat_map(|d| d.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &self.params1.map_or(vec![], |p| {
                [p.0.to_le_bytes(), p.1.to_le_bytes(), p.2.to_le_bytes()].concat()
            })[..],
            &self.params2.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
        ]
        .concat()
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

impl BspRegionFragmentData4Entry {
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

    fn serialize(&self) -> Vec<u8> {
        [
            &self.flags.to_le_bytes()[..],
            &self.params1.to_le_bytes()[..],
            &self.type_field.to_le_bytes()[..],
            &self.params2a.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.params2b.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.name_size.to_le_bytes()[..],
            &self.name.as_bytes()[..],
        ]
        .concat()
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

impl BspRegionFragmentData5Entry {
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

    fn serialize(&self) -> Vec<u8> {
        [
            &self.params1.0.to_le_bytes()[..],
            &self.params1.1.to_le_bytes()[..],
            &self.params1.2.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self.params3.to_le_bytes()[..],
            &self.params4.to_le_bytes()[..],
            &self.params5.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[derive(Debug)]
/// A potentially visible set (PVS) of regions
pub struct BspRegionFragmentPVS {
    /// The number of entries in the `data` field
    size: u16,

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

impl BspRegionFragmentPVS {
    fn parse(input: &[u8]) -> IResult<&[u8], BspRegionFragmentPVS> {
        let (i, size) = le_u16(input)?;
        let (remaining, data) = count(le_u8, size as usize)(i)?;

        Ok((remaining, BspRegionFragmentPVS { size, data }))
    }

    fn serialize(&self) -> Vec<u8> {
        [&self.size.to_le_bytes()[..], &self.data[..]].concat()
    }
}
