use std::any::Any;

use super::{Fragment, FragmentRef, FragmentType, LightSourceReferenceFragment, StringReference};

use nom::multi::count;
use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// A reference to a [LightSourceReferenceFragment].
///
/// **Type ID:** 0x2a
pub struct AmbientLightFragment {
    pub name_reference: StringReference,

    /// The [LightSourceReferenceFragment] reference.
    pub reference: FragmentRef<LightSourceReferenceFragment>,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,

    /// The number of region ids.
    pub region_count: u32,

    /// There are `region_count` region ids here. Each isn’t a fragment reference
    /// per se, but the ID of a 0x22 BSP region fragment. For example, if there are
    /// 100 0x22 BSP Region fragments, then the possible values are in the range 0-99.
    /// This constitutes a list of regions that have the ambient lighting given by the
    /// 0x1C fragment that this fragment references.
    pub regions: Vec<u32>,
}

impl FragmentType for AmbientLightFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2a;
    const TYPE_NAME: &'static str = "AmbientLight";

    fn parse(input: &[u8]) -> IResult<&[u8], AmbientLightFragment> {
        let (i, (name_reference, reference, flags, region_count)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32, le_u32))(input)?;
        let (remaining, regions) = count(le_u32, region_count as usize)(i)?;

        Ok((
            remaining,
            AmbientLightFragment {
                name_reference,
                reference,
                flags,
                region_count,
                regions,
            },
        ))
    }
}

impl Fragment for AmbientLightFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.region_count.to_le_bytes()[..],
            &self
                .regions
                .iter()
                .flat_map(|r| r.to_le_bytes())
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4637-0x2a.frag")[..];
        let frag = AmbientLightFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-52558));
        assert_eq!(frag.flags, 0);
        assert_eq!(frag.region_count, 2905);
        assert_eq!(frag.regions.len(), 2905);
        assert_eq!(frag.regions[0..5], vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4637-0x2a.frag")[..];
        let frag = AmbientLightFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
