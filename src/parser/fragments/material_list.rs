use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, MaterialFragment, StringReference};

use nom::multi::count;
use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
///
/// **Type ID:** 0x31
pub struct MaterialListFragment {
    pub name_reference: StringReference,

    /// _Unknown_ - Must contain 0.
    pub flags: u32,

    /// The number of fragment references this fragment contains.
    pub size1: u32,

    /// `size1` references to [MaterialFragment] fragments.
    pub fragments: Vec<FragmentRef<MaterialFragment>>,
}

impl FragmentType for MaterialListFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x31;

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialListFragment> {
        let (i, (name_reference, flags, size1)) =
            tuple((StringReference::parse, le_u32, le_u32))(input)?;
        let (remaining, fragments) = count(fragment_ref, size1 as usize)(i)?;
        Ok((
            remaining,
            MaterialListFragment {
                name_reference,
                flags,
                size1,
                fragments,
            },
        ))
    }
}

impl Fragment for MaterialListFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .fragments
                .iter()
                .flat_map(|f| f.serialize())
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
