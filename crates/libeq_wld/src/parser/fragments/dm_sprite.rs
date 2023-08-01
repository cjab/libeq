use std::any::Any;

use super::{DmSpriteDef2, Fragment, FragmentParser, FragmentRef, StringReference, WResult};

use nom::number::complete::le_u32;
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// A reference to a [DmSpriteDef2] fragment.
///
/// **Type ID:** 0x2d
pub struct DmSprite {
    pub name_reference: StringReference,

    /// The [DmSpriteDef2] reference.
    pub reference: FragmentRef<DmSpriteDef2>,

    /// _Unknown_ - Apparently must be zero.
    pub params: u32,
}

impl FragmentParser for DmSprite {
    type T = Self;

    const TYPE_ID: u32 = 0x2d;
    const TYPE_NAME: &'static str = "DmSprite";

    fn parse(input: &[u8]) -> WResult<DmSprite> {
        let (remaining, (name_reference, reference, params)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            DmSprite {
                name_reference,
                reference,
                params,
            },
        ))
    }
}

impl Fragment for DmSprite {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.params.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0012-0x2d.frag")[..];
        let frag = DmSprite::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(6));
        assert_eq!(frag.params, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0012-0x2d.frag")[..];
        let frag = DmSprite::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
