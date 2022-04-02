use std::any::Any;

use super::{Fragment, FragmentType, StringReference};

use nom::IResult;

#[derive(Debug)]
/// _Unknown_
///
/// **Type ID:** 0x16
pub struct ZoneUnknownFragment {
    pub name_reference: StringReference,
}

impl FragmentType for ZoneUnknownFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x16;

    fn parse(input: &[u8]) -> IResult<&[u8], ZoneUnknownFragment> {
        let (remaining, name_reference) = StringReference::parse(input)?;
        Ok((remaining, ZoneUnknownFragment { name_reference }))
    }
}

impl Fragment for ZoneUnknownFragment {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name_ref(&self) -> &StringReference {
        &self.name_reference
    }
}
