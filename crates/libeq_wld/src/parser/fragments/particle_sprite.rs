use std::any::Any;

use super::{
    Fragment, FragmentParser, FragmentRef, ParticleSpriteDefFragment, StringReference, WResult,
};

use nom::number::complete::le_u32;
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// A reference to a [ParticleSpriteDefFragment].
///
/// **Type ID:** 0x0d
pub struct ParticleSprite {
    pub name_reference: StringReference,

    /// The [ParticleSpriteDefFragment] reference.
    pub reference: FragmentRef<ParticleSpriteDefFragment>,

    /// _Unknown_.
    pub params1: u32,
}

impl FragmentParser for ParticleSprite {
    type T = Self;

    const TYPE_ID: u32 = 0x0d;
    const TYPE_NAME: &'static str = "ParticleSprite";

    fn parse(input: &[u8]) -> WResult<ParticleSprite> {
        let (remaining, (name_reference, reference, params1)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            ParticleSprite {
                name_reference,
                reference,
                params1,
            },
        ))
    }
}

impl Fragment for ParticleSprite {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.params1.to_le_bytes()[..],
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
    fn it_has_a_known_name_reference() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!(
            "../../../fixtures/fragments/wldcom/particle-sprite-0001-0x0d.frag"
        )[..];
        let frag = ParticleSprite::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0x0));
        assert_eq!(frag.reference, FragmentRef::new(0x1));
        assert_eq!(frag.params1, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!(
            "../../../fixtures/fragments/wldcom/particle-sprite-0001-0x0d.frag"
        )[..];
        let frag = ParticleSprite::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
