use std::any::Any;

use nom::number::complete::{le_f32, le_u32};

use super::{Fragment, FragmentParser, StringReference, WResult, BlitSpriteDefinitionFragment, FragmentRef};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Unknown
///
/// **Type ID:** 0x34
pub struct Unknown0x34Fragment {
    pub name_reference: StringReference,
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
    pub unknown_5: u32,
    pub unknown_6: u32,
    pub unknown_7: u32,
    pub unknown_8: u32,
    pub unknown_9: u32,
    pub unknown_10: u32,
    pub unknown_11: f32,
    pub unknown_12: f32,
    pub unknown_13: u32,
    pub unknown_14: f32,
    pub unknown_15: u32,
    pub unknown_16: f32,
    pub unknown_17: u32,
    pub unknown_18: u32,
    pub unknown_19: f32,
    pub unknown_20: f32,
    pub blitsprite: FragmentRef<BlitSpriteDefinitionFragment>
}

impl FragmentParser for Unknown0x34Fragment {
    type T = Self;

    const TYPE_ID: u32 = 0x34;
    const TYPE_NAME: &'static str = "Unknown0x34";

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, unknown_1) = le_u32(i)?;
        let (i, unknown_2) = le_u32(i)?;
        let (i, unknown_3) = le_u32(i)?;
        let (i, unknown_4) = le_u32(i)?;
        let (i, unknown_5) = le_u32(i)?;
        let (i, unknown_6) = le_u32(i)?;
        let (i, unknown_7) = le_u32(i)?;
        let (i, unknown_8) = le_u32(i)?;
        let (i, unknown_9) = le_u32(i)?;
        let (i, unknown_10) = le_u32(i)?;
        let (i, unknown_11) = le_f32(i)?;
        let (i, unknown_12) = le_f32(i)?;
        let (i, unknown_13) = le_u32(i)?;
        let (i, unknown_14) = le_f32(i)?;
        let (i, unknown_15) = le_u32(i)?;
        let (i, unknown_16) = le_f32(i)?;
        let (i, unknown_17) = le_u32(i)?;
        let (i, unknown_18) = le_u32(i)?;
        let (i, unknown_19) = le_f32(i)?;
        let (i, unknown_20) = le_f32(i)?;
        let (i, blitsprite) = FragmentRef::<BlitSpriteDefinitionFragment>::parse(i)?;

        Ok((
            i,
            Self {
                name_reference,
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
                unknown_6,
                unknown_7,
                unknown_8,
                unknown_9,
                unknown_10,
                unknown_11,
                unknown_12,
                unknown_13,
                unknown_14,
                unknown_15,
                unknown_16,
                unknown_17,
                unknown_18,
                unknown_19,
                unknown_20,
                blitsprite,
            },
        ))
    }
}

impl Fragment for Unknown0x34Fragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.unknown_1.to_le_bytes()[..],
            &self.unknown_2.to_le_bytes()[..],
            &self.unknown_3.to_le_bytes()[..],
            &self.unknown_4.to_le_bytes()[..],
            &self.unknown_5.to_le_bytes()[..],
            &self.unknown_6.to_le_bytes()[..],
            &self.unknown_7.to_le_bytes()[..],
            &self.unknown_8.to_le_bytes()[..],
            &self.unknown_9.to_le_bytes()[..],
            &self.unknown_10.to_le_bytes()[..],
            &self.unknown_11.to_le_bytes()[..],
            &self.unknown_12.to_le_bytes()[..],
            &self.unknown_13.to_le_bytes()[..],
            &self.unknown_14.to_le_bytes()[..],
            &self.unknown_15.to_le_bytes()[..],
            &self.unknown_16.to_le_bytes()[..],
            &self.unknown_17.to_le_bytes()[..],
            &self.unknown_18.to_le_bytes()[..],
            &self.unknown_19.to_le_bytes()[..],
            &self.unknown_20.to_le_bytes()[..],
            &self.blitsprite.into_bytes()[..],
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
