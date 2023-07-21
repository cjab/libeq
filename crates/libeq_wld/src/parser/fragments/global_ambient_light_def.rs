use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// The first fragment has a single field. A name reference
/// that always has a value of 0xff000000.
pub struct GlobalAmbientLightDef {
    pub name_reference: StringReference,
}

impl FragmentParser for GlobalAmbientLightDef {
    type T = Self;

    const TYPE_ID: u32 = 0x35;
    const TYPE_NAME: &'static str = "GlobalAmbientLightDef";

    fn parse(input: &[u8]) -> WResult<GlobalAmbientLightDef> {
        let (remainder, name_reference) = StringReference::parse(input)?;
        Ok((remainder, GlobalAmbientLightDef { name_reference }))
    }
}

impl Fragment for GlobalAmbientLightDef {
    fn into_bytes(&self) -> Vec<u8> {
        [&self.name_reference.into_bytes()[..]].concat()
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
    fn it_has_a_known_name_reference() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0000-0x35.frag")[..];
        let frag = GlobalAmbientLightDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0xff000000));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0000-0x35.frag")[..];
        let frag = GlobalAmbientLightDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
