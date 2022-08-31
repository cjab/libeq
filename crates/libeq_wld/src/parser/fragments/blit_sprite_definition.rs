use std::any::Any;

use nom::number::complete::{le_i32, le_u32};

use super::{Fragment, FragmentParser, StringReference, WResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// BLITSPRITEDEFINITION
///
/// **Type ID:** 0x26
pub struct BlitSpriteDefinitionFragment {
    pub name_reference: StringReference,
    pub flags: BlitSpriteDefFlags,
    pub blit_sprite_reference: u32,
    pub unknown: i32,
}

impl FragmentParser for BlitSpriteDefinitionFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x26;
    const TYPE_NAME: &'static str = "BlitSpriteDefinition";

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = BlitSpriteDefFlags::parse(i)?;
        let (i, blit_sprite_reference) = le_u32(i)?;
        let (i, unknown) = le_i32(i)?;

        Ok((
            i,
            Self {
                name_reference,
                flags,
                blit_sprite_reference,
                unknown,
            },
        ))
    }
}

impl Fragment for BlitSpriteDefinitionFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct BlitSpriteDefFlags(u32);

impl BlitSpriteDefFlags {
    const TRANSPARENT: u32 = 0x100;

    pub fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub fn parse(input: &[u8]) -> WResult<Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn transparent(&self) -> bool {
        self.0 & Self::TRANSPARENT == Self::TRANSPARENT
    }
}
