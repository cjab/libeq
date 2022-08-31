use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// **Type ID:** 0x17
pub struct PolygonAnimationFragment {
    pub name_reference: StringReference,

    /// _Unknown_ - Usually contains 0.1 // NOTE: WLD ref lists a float here, but I think it was a typo.
    //pub params1: f32,

    /// _Unknown_
    /// * bit 0 - If unset `params2` must be 1.0
    pub flags: u32,

    /// The number of `entries1` entries.
    pub size1: u32,

    /// The number of `entries2` entries.
    pub size2: u32,

    /// _Unknown_
    pub params1: f32,

    /// _Unknown_ - Usually contains 1.0
    pub params2: f32,

    /// _Unknown_ - There are size1 of these.
    pub entries1: Vec<(f32, f32, f32)>,
    /// _Unknown_ - There are size2 of these.
    ///
    /// Tuple is as follows:
    /// (number of entries in data, data vec)
    ///
    /// The data appears to be indices into the X, Y, Z entries above.
    pub entries2: Vec<(u32, Vec<u32>)>,
}

impl FragmentParser for PolygonAnimationFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x17;
    const TYPE_NAME: &'static str = "PolygonAnimation";

    fn parse(input: &[u8]) -> WResult<PolygonAnimationFragment> {
        let (i, (name_reference, flags, size1, size2, params1, params2)) = tuple((
            StringReference::parse,
            //le_f32,
            le_u32,
            le_u32,
            le_u32,
            le_f32,
            le_f32,
        ))(input)?;

        let (i, entries1) = count(tuple((le_f32, le_f32, le_f32)), size1 as usize)(i)?;

        let entry2 = |input| {
            let (i, entry_size) = le_u32(input)?;
            let (i, entries) = count(le_u32, entry_size as usize)(i)?;
            Ok((i, (entry_size, entries)))
        };

        let (remaining, entries2) = count(entry2, size2 as usize)(i)?;

        Ok((
            remaining,
            PolygonAnimationFragment {
                name_reference,
                flags,
                size1,
                size2,
                params1,
                params2,
                entries1,
                entries2,
            },
        ))
    }
}

impl Fragment for PolygonAnimationFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            // &self.params1.to_le_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self.size2.to_le_bytes()[..],
            &self.params1.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self
                .entries1
                .iter()
                .flat_map(|e| [e.0.to_le_bytes(), e.1.to_le_bytes(), e.2.to_le_bytes()].concat())
                .collect::<Vec<_>>()[..],
            &self
                .entries2
                .iter()
                .flat_map(|e| {
                    [
                        &e.0.to_le_bytes()[..],
                        &e.1.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>()[..],
                    ]
                    .concat()
                })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/1417-0x17.frag")[..];
        let frag = PolygonAnimationFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-14003));
        assert_eq!(frag.flags, 0x1);
        assert_eq!(frag.size1, 8);
        assert_eq!(frag.size2, 6);
        assert_eq!(frag.params1, 8.902873);
        assert_eq!(frag.params2, 1.0);
        assert_eq!(frag.entries1.len(), 8);
        assert_eq!(frag.entries1[0], (-0.06475502, -1.2741688, -6.6802998));
        assert_eq!(frag.entries2.len(), 6);
        assert_eq!(frag.entries2[0], (4, vec![3, 2, 0, 1]));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/1417-0x17.frag")[..];
        let frag = PolygonAnimationFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
