use std::any::Any;

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Fragment, FragmentParser, StringReference, WResult};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// 4DSPRITEDEF fragment
///
/// **Type ID:** 0x0a
pub struct Sprite4DDef {
    pub name_reference: StringReference,

    pub flags: Sprite4DDefFlags,

    /// NUMFRAMES %d
    pub num_frames: u32,

    /// SPHERE, SPHERELIST, or POLYHEDRON reference
    pub polygon_fragment: u32,

    /// CENTEROFFSET %f %f %f
    pub center_offset: Option<(f32, f32, f32)>,

    /// BOUNDINGRADIUS %f
    pub bounding_radius: Option<f32>,

    /// CURRENTFRAME %d
    pub current_frame: Option<u32>,

    /// SLEEP %d
    pub sleep: Option<u32>,

    /// SIMPLESPRITE, 2DSPRITE, 3DSPRITE, 4DSPRITE, PARTICLESPRITE, COMPOSITESPRITE, HIERARCHICALSPRITE, BLITSPRITE or NULLSPRITE references
    pub sprite_fragments: Option<Vec<u32>>,
}

impl FragmentParser for Sprite4DDef {
    type T = Self;

    const TYPE_ID: u32 = 0x0a;
    const TYPE_NAME: &'static str = "Sprite4DDef";

    fn parse(input: &[u8]) -> WResult<Sprite4DDef> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = Sprite4DDefFlags::parse(i)?;
        let (i, num_frames) = le_u32(i)?;
        let (i, polygon_fragment) = le_u32(i)?;

        let (i, center_offset) = if flags.has_center_offset() {
            tuple((le_f32, le_f32, le_f32))(i).map(|(i, p3)| (i, Some(p3)))?
        } else {
            (i, None)
        };

        let (i, bounding_radius) = if flags.has_bounding_radius() {
            le_f32(i).map(|(i, p4)| (i, Some(p4)))?
        } else {
            (i, None)
        };

        let (i, current_frame) = if flags.has_current_frame() {
            le_u32(i).map(|(i, p5)| (i, Some(p5)))?
        } else {
            (i, None)
        };

        let (i, sleep) = if flags.has_sleep() {
            le_u32(i).map(|(i, p6)| (i, Some(p6)))?
        } else {
            (i, None)
        };

        let (i, sprite_fragments) = if flags.has_sprites() {
            count(le_u32, num_frames as usize)(i).map(|(rem, v)| (rem, Some(v)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            Sprite4DDef {
                name_reference,
                flags,
                num_frames,
                polygon_fragment,
                center_offset,
                bounding_radius,
                current_frame,
                sleep,
                sprite_fragments,
            },
        ))
    }
}

impl Fragment for Sprite4DDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.num_frames.to_le_bytes()[..],
            &self.polygon_fragment.to_le_bytes()[..],
            &self.center_offset.map_or(vec![], |c| {
                [c.0.to_le_bytes(), c.1.to_le_bytes(), c.2.to_le_bytes()].concat()
            })[..],
            &self
                .bounding_radius
                .map_or(vec![], |b| b.to_le_bytes().to_vec())[..],
            &self
                .current_frame
                .map_or(vec![], |c| c.to_le_bytes().to_vec())[..],
            &self.sleep.map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
            &self
                .sprite_fragments
                .as_ref()
                .map_or(vec![], |o| o.iter().flat_map(|v| v.to_le_bytes()).collect())[..],
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
pub struct Sprite4DDefFlags(u32);

impl Sprite4DDefFlags {
    const HAS_CENTER_OFFSET: u32 = 0x01;
    const HAS_BOUNDING_RADIUS: u32 = 0x02;
    const HAS_CURRENT_FRAME: u32 = 0x04;
    const HAS_SLEEP: u32 = 0x08;
    const HAS_SPRITES: u32 = 0x10;
    const SKIP_FRAMES: u32 = 0x40;

    fn parse(input: &[u8]) -> WResult<Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_center_offset(&self) -> bool {
        self.0 & Self::HAS_CENTER_OFFSET == Self::HAS_CENTER_OFFSET
    }

    pub fn has_bounding_radius(&self) -> bool {
        self.0 & Self::HAS_BOUNDING_RADIUS == Self::HAS_BOUNDING_RADIUS
    }

    pub fn has_current_frame(&self) -> bool {
        self.0 & Self::HAS_CURRENT_FRAME == Self::HAS_CURRENT_FRAME
    }

    pub fn has_sleep(&self) -> bool {
        self.0 & Self::HAS_SLEEP == Self::HAS_SLEEP
    }

    pub fn has_sprites(&self) -> bool {
        self.0 & Self::HAS_SPRITES == Self::HAS_SPRITES
    }

    pub fn skip_frames(&self) -> bool {
        self.0 & Self::SKIP_FRAMES == Self::SKIP_FRAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data =
            &include_bytes!("../../../fixtures/fragments/wldcom/4dspritedef-0004-0x0a.frag")[..];
        let (remaining, frag) = Sprite4DDef::parse(data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(-28));
        assert_eq!(frag.flags, Sprite4DDefFlags(0x5f));
        assert_eq!(frag.num_frames, 3);
        assert_eq!(frag.polygon_fragment, 4);
        assert_eq!(frag.center_offset, Some((1.1, 1.2, 1.3)));
        assert_eq!(frag.bounding_radius, Some(0.13));
        assert_eq!(frag.current_frame, Some(1));
        assert_eq!(frag.sleep, Some(4));
        assert_eq!(frag.sprite_fragments, Some(vec![1, 2, 3]));

        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn it_serializes() {
        let data =
            &include_bytes!("../../../fixtures/fragments/wldcom/4dspritedef-0004-0x0a.frag")[..];
        let frag = Sprite4DDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
