use std::any::Any;

use super::{Fragment, FragmentType, StringReference};

use nom::number::complete::le_i32;
use nom::IResult;

#[derive(Debug)]
/// _Unknown_
///
/// **Type ID:** 0x16
pub struct ZoneUnknownFragment {
    pub name_reference: StringReference,

    pub unknown: i32,
}

impl FragmentType for ZoneUnknownFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x16;
    const TYPE_NAME: &'static str = "ZoneUnknown";

    fn parse(input: &[u8]) -> IResult<&[u8], ZoneUnknownFragment> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (remaining, unknown) = le_i32(i)?;
        Ok((
            remaining,
            ZoneUnknownFragment {
                name_reference,
                unknown,
            },
        ))
    }
}

impl Fragment for ZoneUnknownFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.unknown.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4640-0x16.frag")[..];
        let frag = ZoneUnknownFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4640-0x16.frag")[..];
        let frag = ZoneUnknownFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
