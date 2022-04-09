use std::any::Any;

use super::{Fragment, FragmentParser, StringReference};

use nom::multi::count;
use nom::number::complete::{le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// This fragment lets you flag certain regions (as defined by 0x22 BSP Region fragments)
/// in a particular way. The flagging is done by setting the name of this fragment to a
/// particular “magic” value.
///
/// The possible values are:
///
/// * WT_ZONE ...............Flag all regions in the list as underwater regions.
/// * LA_ZONE ...............Flag all regions in the list as lava regions.
/// * DRP_ZONE ..............Flag all regions in the list as PvP regions.
/// * DRNTP##########_ZONE...Flag all regions in the list as zone point regions.
///                          The ####’s are actually numbers and hyphens that somehow tell
///                          the client the zone destination. This method of setting zone
///                          points may or may not be obsolete.
///
/// **Type ID:** 0x29
pub struct RegionFlagFragment {
    pub name_reference: StringReference,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,

    /// The number of region ids.
    pub region_count: u32,

    /// There are `region_count` regions. Each isn’t a fragment reference per se, but the
    /// ID of a 0x22 BSP region fragment. For example, if there are 100 0x22 BSP Region
    /// fragments, then the possible values are in the range 0-99. This constitutes a
    /// list of regions that are to be flagged in the particular way.
    pub regions: Vec<u32>,

    /// The number of bytes following in the `data2` field.
    pub size2: u32,

    /// An encoded string. An alternate way of using this fragment is to call this fragment
    /// Z####_ZONE, where #### is a four- digit number starting with zero. Then Data2 would
    /// contain a “magic” string that told the client what was special about the included
    /// regions (e.g. WTN__01521000000000000000000000___000000000000). This field is padded
    /// with nulls to make it end on a DWORD boundary.
    pub data2: Vec<u8>,
}

impl FragmentParser for RegionFlagFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x29;
    const TYPE_NAME: &'static str = "RegionFlag";

    fn parse(input: &[u8]) -> IResult<&[u8], RegionFlagFragment> {
        let (i, (name_reference, flags, region_count)) =
            tuple((StringReference::parse, le_u32, le_u32))(input)?;
        let (i, (regions, size2)) = tuple((count(le_u32, region_count as usize), le_u32))(i)?;

        let padding = (4 - size2 % 4) % 4;
        let size2_with_padding = size2 + padding;
        let (remaining, data2) = count(le_u8, size2_with_padding as usize)(i)?;

        Ok((
            remaining,
            RegionFlagFragment {
                name_reference,
                flags,
                region_count,
                regions,
                size2,
                data2,
            },
        ))
    }
}

impl Fragment for RegionFlagFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.region_count.to_le_bytes()[..],
            &self
                .regions
                .iter()
                .flat_map(|r| r.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &self.size2.to_le_bytes()[..],
            &self
                .data2
                .iter()
                .flat_map(|d| d.to_le_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4642-0x29.frag")[..];
        let frag = RegionFlagFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-52603));
        assert_eq!(frag.flags, 0x0);
        assert_eq!(frag.region_count, 2);
        assert_eq!(frag.regions, vec![2859, 2865]);
        assert_eq!(frag.size2, 0);
        assert_eq!(frag.data2, vec![]);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4642-0x29.frag")[..];
        let frag = RegionFlagFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}