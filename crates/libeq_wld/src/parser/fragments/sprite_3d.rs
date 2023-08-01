use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, Sprite3DDef, StringReference, WResult};

use nom::number::complete::le_u32;
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// A reference to a [Sprite3DDef].
///
/// **Type ID:** 0x09
pub struct Sprite3D {
    pub name_reference: StringReference,

    /// The [Sprite3DDef] reference.
    pub reference: FragmentRef<Sprite3DDef>,

    /// _Unknown_ Seems to always contain 0.
    pub flags: u32,
}

impl FragmentParser for Sprite3D {
    type T = Self;

    const TYPE_ID: u32 = 0x09;
    const TYPE_NAME: &'static str = "Sprite3D";

    fn parse(input: &[u8]) -> WResult<Sprite3D> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            Sprite3D {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for Sprite3D {
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4638-0x09.frag")[..];
        let frag = Sprite3D::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(1730));
        assert_eq!(frag.flags, 0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4638-0x09.frag")[..];
        let frag = Sprite3D::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
