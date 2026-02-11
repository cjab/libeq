use std::any::Any;

use super::{BmInfo, Fragment, FragmentParser, FragmentRef, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::le_u32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// This fragment represents an entire texture rather than merely a bitmap used by that
/// texture. The conceptual difference from [BmInfo] fragments is that textures
/// may be animated; the [SimpleSpriteDef] fragment represents the entire texture
/// including all bitmaps that it uses whereas a [BmInfo] fragment would
/// represent only a single bitmap in the animated sequence.
///
/// **Type ID:** 0x04
pub struct SimpleSpriteDef {
    pub name_reference: StringReference,

    /// Most flags are _unknown_ however:
    /// * bit 3 - If set texture is animated (has more than one [BmInfo] reference.
    /// This also means that a `params1` field exists.
    /// * bit 4 - If set a `params2` field exists. This _seems_ to always be set.
    pub flags: SimpleSpriteDefFlags,

    /// The number of [BmInfo] references.
    pub frame_count: u32,

    /// Only present if bit `has_current_frame` in `flags` is set.
    pub current_frame: Option<u32>,

    /// Only present if `sleep` in `flags` is set.
    pub sleep: Option<u32>,

    /// One or more references to [BmInfo] fragments. For most textures this will
    /// be a single reference but animated textures will reference multiple.
    pub frame_references: Vec<FragmentRef<BmInfo>>,
}

impl FragmentParser for SimpleSpriteDef {
    type T = Self;

    const TYPE_ID: u32 = 0x04;
    const TYPE_NAME: &'static str = "SimpleSpriteDef";

    fn parse(input: &[u8]) -> WResult<'_, SimpleSpriteDef> {
        let (i, (name_reference, flags, frame_count)) =
            (StringReference::parse, SimpleSpriteDefFlags::parse, le_u32).parse(input)?;

        //TODO: Is this a thing? Find an example.
        let (i, _current_frame) = if flags.has_current_frame() {
            let (i, current_frame) = le_u32(i)?;
            (i, Some(current_frame))
        } else {
            (i, None)
        };
        let current_frame = None;

        let (i, sleep) = if flags.is_animated() && flags.has_sleep() {
            let (i, sleep) = le_u32(i)?;
            (i, Some(sleep))
        } else {
            (i, None)
        };

        let (remaining, frame_references) =
            count(FragmentRef::parse, frame_count as usize).parse(i)?;

        Ok((
            remaining,
            SimpleSpriteDef {
                name_reference,
                flags,
                frame_count,
                current_frame,
                sleep,
                frame_references,
            },
        ))
    }
}

impl Fragment for SimpleSpriteDef {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
            &self.flags.to_bytes()[..],
            &self.frame_count.to_le_bytes()[..],
            &self
                .current_frame
                .map_or(vec![], |c| c.to_le_bytes().to_vec())[..],
            &self.sleep.map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
            &self
                .frame_references
                .iter()
                .flat_map(|f| f.to_bytes())
                .collect::<Vec<_>>()[..],
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
pub struct SimpleSpriteDefFlags(pub u32);

impl SimpleSpriteDefFlags {
    const SKIP_FRAMES: u32 = 0x02;
    const IS_ANIMATED: u32 = 0x08;
    const HAS_SLEEP: u32 = 0x10;
    const HAS_CURRENT_FRAME: u32 = 0x20;

    fn parse(input: &[u8]) -> WResult<'_, SimpleSpriteDefFlags> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, SimpleSpriteDefFlags(raw_flags)))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn skip_frames(&self) -> bool {
        self.0 & Self::SKIP_FRAMES == Self::SKIP_FRAMES
    }

    pub fn is_animated(&self) -> bool {
        self.0 & Self::IS_ANIMATED == Self::IS_ANIMATED
    }

    pub fn has_sleep(&self) -> bool {
        self.0 & Self::HAS_SLEEP == Self::HAS_SLEEP
    }

    pub fn has_current_frame(&self) -> bool {
        self.0 & Self::HAS_CURRENT_FRAME == Self::HAS_CURRENT_FRAME
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0002-0x04.frag")[..];
        let frag = SimpleSpriteDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0xfffffff8));
        assert_eq!(frag.flags.0, 0x10);
        //FIXME: This seems wrong
        //assert_eq!(frag.flags.has_sleep(), true);
        assert_eq!(frag.flags.has_current_frame(), false);
        assert_eq!(frag.flags.skip_frames(), false);
        assert_eq!(frag.flags.is_animated(), false);
        assert_eq!(frag.frame_count, 0x1);
        assert_eq!(frag.current_frame, None);
        assert_eq!(frag.sleep, None);
        assert_eq!(frag.frame_references.len(), 1);
        assert_eq!(frag.frame_references[0], FragmentRef::new(0x02));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0002-0x04.frag")[..];
        let frag = SimpleSpriteDef::parse(data).unwrap().1;

        assert_eq!(&frag.to_bytes()[..], data);
    }
}
