use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, StringHash, TextureImagesFragment};

use nom::multi::count;
use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// This fragment represents an entire texture rather than merely a bitmap used by that
/// texture. The conceptual difference from [TextureImagesFragment] fragments is that textures
/// may be animated; the [TextureFragment] fragment represents the entire texture
/// including all bitmaps that it uses whereas a [TextureImagesFragment] fragment would
/// represent only a single bitmap in the animated sequence.
///
/// **Type ID:** 0x04
pub struct TextureFragment {
    /// Most flags are _unknown_ however:
    /// * bit 3 - If set texture is animated (has more than one [TextureImagesFragment] reference.
    /// This also means that a `params1` field exists.
    /// * bit 4 - If set a `params2` field exists. This _seems_ to always be set.
    pub flags: TextureFragmentFlags,

    /// The number of [TextureImagesFragment] references.
    pub frame_count: u32,

    /// Only present if bit `has_current_frame` in `flags` is set.
    pub current_frame: Option<u32>,

    /// Only present if `sleep` in `flags` is set.
    pub sleep: Option<u32>,

    /// One or more references to [TextureImagesFragment] fragments. For most textures this will
    /// be a single reference but animated textures will reference multiple.
    pub frame_references: Vec<FragmentRef<TextureImagesFragment>>,
}

impl FragmentType for TextureFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x04;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureFragment> {
        let (i, (flags, frame_count)) = tuple((TextureFragmentFlags::parse, le_u32))(input)?;

        // TODO: Do these fields even really exist?
        let current_frame = None;
        let sleep = None;
        let (remaining, frame_references) = count(fragment_ref, frame_count as usize)(i)?;

        Ok((
            remaining,
            TextureFragment {
                flags,
                frame_count,
                current_frame,
                sleep,
                frame_references,
            },
        ))
    }
}

impl Fragment for TextureFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.flags.serialize()[..],
            &self.frame_count.to_le_bytes()[..],
            &self
                .current_frame
                .map_or(vec![], |c| c.to_le_bytes().to_vec())[..],
            &self.sleep.map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
            &self
                .frame_references
                .iter()
                .flat_map(|f| f.serialize())
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self, string_hash: &StringHash) -> String {
        String::new()
    }
}

#[derive(Debug)]
pub struct TextureFragmentFlags(pub u32);

impl TextureFragmentFlags {
    const SKIP_FRAMES: u32 = 0x02;
    const IS_ANIMATED: u32 = 0x08;
    const HAS_SLEEP: u32 = 0x10;
    const HAS_CURRENT_FRAME: u32 = 0x10;

    fn parse(input: &[u8]) -> IResult<&[u8], TextureFragmentFlags> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, TextureFragmentFlags(raw_flags)))
    }

    fn serialize(&self) -> Vec<u8> {
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
