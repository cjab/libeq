use std::any::Any;

use super::{Fragment, FragmentType, StringHash};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// **Type ID:** 0x17
pub struct PolygonAnimationFragment {
    /// _Unknown_ - Usually contains 0.1
    pub params1: f32,

    /// _Unknown_
    /// * bit 0 - If unset `params2` must be 1.0
    pub flags: u32,

    /// The number of `entries1` entries.
    pub size1: u32,

    /// The number of `entries2` entries.
    pub size2: u32,

    /// _Unknown_
    pub params2: f32,

    /// _Unknown_ - Usually contains 1.0
    pub params3: f32,

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

impl FragmentType for PolygonAnimationFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x17;

    fn parse(input: &[u8]) -> IResult<&[u8], PolygonAnimationFragment> {
        let (i, (params1, flags, size1, size2, params2, params3)) =
            tuple((le_f32, le_u32, le_u32, le_u32, le_f32, le_f32))(input)?;

        let entry2 = |input| {
            let (i, entry_size) = le_u32(input)?;
            let (i, entries) = count(le_u32, entry_size as usize)(i)?;
            Ok((i, (entry_size, entries)))
        };

        let (remaining, (entries1, entries2)) = tuple((
            count(tuple((le_f32, le_f32, le_f32)), size1 as usize),
            count(entry2, size2 as usize),
        ))(i)?;

        Ok((
            remaining,
            PolygonAnimationFragment {
                params1,
                flags,
                size1,
                size2,
                params2,
                params3,
                entries1,
                entries2,
            },
        ))
    }
}

impl Fragment for PolygonAnimationFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.params1.to_le_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self.size2.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self.params3.to_le_bytes()[..],
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

    fn name(&self, string_hash: &StringHash) -> String {
        String::new()
    }
}
