use std::any::Any;

use super::{Fragment, FragmentType, StringReference};

use nom::IResult;

#[derive(Debug)]
/// There are no fields.
///
/// **Type ID:** 0x35
pub struct FirstFragment {
    pub name_reference: StringReference,
}

impl FragmentType for FirstFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x35;

    fn parse(input: &[u8]) -> IResult<&[u8], FirstFragment> {
        // TODO: Does this actually have a name reference?
        let name_reference = StringReference::new(0);
        Ok((input, FirstFragment { name_reference }))
    }
}

impl Fragment for FirstFragment {
    fn serialize(&self) -> Vec<u8> {
        [&self.name_reference.serialize()[..]].concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name_ref(&self) -> &StringReference {
        &self.name_reference
    }
}
