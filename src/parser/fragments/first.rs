use std::any::Any;

use super::{Fragment, FragmentType, StringHash};

use nom::IResult;

#[derive(Debug)]
/// There are no fields.
///
/// **Type ID:** 0x35
pub struct FirstFragment {}

impl FragmentType for FirstFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x35;

    fn parse(input: &[u8]) -> IResult<&[u8], FirstFragment> {
        Ok((input, FirstFragment {}))
    }
}

impl Fragment for FirstFragment {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self, string_hash: &StringHash) -> String {
        String::new()
    }
}
