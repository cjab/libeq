use std::any::Any;

use nom::number::complete::{le_i32, le_u32};

use super::{Fragment, FragmentParser, StringReference, WResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// BLITSPRITE TAG
///
/// **Type ID:** 0x27
pub struct BlitSprite {
    pub name_reference: StringReference,
    pub blit_sprite_reference: u32,
    pub unknown: i32,
}

impl FragmentParser for BlitSprite {
    type T = Self;

    const TYPE_ID: u32 = 0x27;
    const TYPE_NAME: &'static str = "BlitSprite";

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, blit_sprite_reference) = le_u32(i)?;
        let (i, unknown) = le_i32(i)?;

        Ok((
            i,
            Self {
                name_reference,
                blit_sprite_reference,
                unknown,
            },
        ))
    }
}

impl Fragment for BlitSprite {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
            &self.blit_sprite_reference.to_le_bytes()[..],
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

    fn type_id(&self) -> u32 {
        Self::TYPE_ID
    }
}
