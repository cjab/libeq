use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, Sprite2DDef, StringReference, WResult};

use nom::Parser;
use nom::number::complete::le_u32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [Sprite2D].
///
/// **Type ID:** 0x07
pub struct Sprite2D {
    pub name_reference: StringReference,

    /// The [Sprite2DDef] reference.
    pub reference: FragmentRef<Sprite2DDef>,

    /// _Unknown_ Seems to always contain 0.
    pub flags: u32,
}

impl FragmentParser for Sprite2D {
    type T = Self;

    const TYPE_ID: u32 = 0x07;
    const TYPE_NAME: &'static str = "Sprite2D";

    fn parse(input: &[u8]) -> WResult<'_, Sprite2D> {
        let (remaining, (name_reference, reference, flags)) =
            (StringReference::parse, FragmentRef::parse, le_u32).parse(input)?;
        Ok((
            remaining,
            Sprite2D {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for Sprite2D {
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
        let data = &include_bytes!("../../../fixtures/fragments/gequip/2224-0x07.frag")[..];
        let frag = Sprite2D::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x07f0));
        assert_eq!(frag.flags, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/2224-0x07.frag")[..];
        let frag = Sprite2D::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
