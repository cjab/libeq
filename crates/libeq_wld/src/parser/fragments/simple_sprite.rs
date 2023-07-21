use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, StringReference, SimpleSpriteDef, WResult};

use nom::number::complete::le_u32;
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [SimpleSpriteDef] fragment.
///
/// **Type ID:** 0x05
pub struct SimpleSprite {
    pub name_reference: StringReference,

    /// The [SimpleSpriteDef] reference.
    pub reference: FragmentRef<SimpleSpriteDef>,

    /// _Unknown_ - Seems to always contain 0x50.
    pub flags: u32,
}

impl FragmentParser for SimpleSprite {
    type T = Self;

    const TYPE_ID: u32 = 0x05;
    const TYPE_NAME: &'static str = "SimpleSprite";

    fn parse(input: &[u8]) -> WResult<SimpleSprite> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            SimpleSprite {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for SimpleSprite {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0003-0x05.frag")[..];
        let frag = SimpleSprite::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0x0));
        assert_eq!(frag.reference, FragmentRef::new(0x3));
        assert_eq!(frag.flags, 0x50);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0003-0x05.frag")[..];
        let frag = SimpleSprite::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
