use std::any::Any;
use std::fmt;

use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// **Type ID:** 0x1b
pub struct LightDef {
    pub name_reference: StringReference,

    /// Windcatcher:
    /// _Unknown_
    /// * bit 1 - Usually 1 when dealing with placed light sources.
    /// * bit 2 - Usually 1.
    /// * bit 3 - Usually 1 when dealing with placed light source.
    ///           If bit 4 is set then `params3b` only exists if
    ///           this bit is also set (not sure about this).
    /// * bit 4 - If unset `params3a` exists but `params3b` and `red`, `green` and `blue` don't exist.
    ///           Otherwise, `params3a` doesn't exist but `params3b` and `red`, `green` and `blue` do exist.
    ///           This flag seems to determine whether the light is just a simple white light
    ///           or a light with its own color values.
    pub flags: LightDefFlags,

    /// Windcatcher:
    /// _Unknown_ - Usually contains 1
    /// NEW:
    /// Corresponds to FRAMECOUNT in LIGHTDEFINITION.
    pub frame_count: u32,

    /// Windcatcher:
    /// _Unknown_ - Usually contains 1.
    /// NEW:
    /// Corresponds to CURRENTFRAME in LIGHTDEFINITION.
    pub current_frame: Option<u32>,

    /// Windcatcher:
    /// _Unknown_ - Usually contains 1
    /// NEW:
    /// Corresponds to SLEEP in LIGHTDEFINITION.
    pub sleep: Option<u32>,

    /// Windcatcher:
    /// _Unknown_ - Usually contains 200 (attenuation?).
    /// NEW:
    /// Corresponds to LIGHTLEVELS in LIGHTDEFINITION.
    pub light_levels: Option<Vec<f32>>,

    /// Red, Green, Blue components, scaled from 0 to 1.
    pub colors: Option<Vec<(f32, f32, f32)>>,
}

impl FragmentParser for LightDef {
    type T = Self;

    const TYPE_ID: u32 = 0x1b;
    const TYPE_NAME: &'static str = "LightDef";

    fn parse(input: &[u8]) -> WResult<'_, LightDef> {
        let (i, (name_reference, flags, frame_count)) =
            (StringReference::parse, LightDefFlags::parse, le_u32).parse(input)?;

        let (i, current_frame) = if flags.has_current_frame() {
            le_u32(i).map(|(i, flags)| (i, Some(flags)))?
        } else {
            (i, None)
        };

        let (i, sleep) = if flags.has_sleep() {
            le_u32(i).map(|(i, sleep)| (i, Some(sleep)))?
        } else {
            (i, None)
        };

        let (i, light_levels) = if flags.has_light_levels() {
            count(le_f32, frame_count as usize)
                .parse(i)
                .map(|(i, light_levels)| (i, Some(light_levels)))?
        } else {
            (i, None)
        };

        let (remaining, colors) = if flags.has_color() {
            count((le_f32, le_f32, le_f32), frame_count as usize)
                .parse(i)
                .map(|(i, colors)| (i, Some(colors)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            LightDef {
                name_reference,
                flags,
                frame_count,
                current_frame,
                sleep,
                light_levels,
                colors,
            },
        ))
    }
}

impl Fragment for LightDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.frame_count.to_le_bytes()[..],
            &self
                .current_frame
                .map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.sleep.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.light_levels.as_ref().map_or(vec![], |p| {
                p.iter().flat_map(|x| x.to_le_bytes().to_vec()).collect()
            })[..],
            &self.colors.as_ref().map_or(vec![], |p| {
                p.iter()
                    .flat_map(|(r, g, b)| {
                        [r.to_le_bytes(), g.to_le_bytes(), b.to_le_bytes()].concat()
                    })
                    .collect()
            })[..],
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
#[derive(PartialEq)]
pub struct LightDefFlags(u32);

impl LightDefFlags {
    const HAS_CURRENT_FRAME: u32 = 0x01;
    const HAS_SLEEP: u32 = 0x02;
    const HAS_LIGHT_LEVELS: u32 = 0x04;
    const SKIP_FRAMES: u32 = 0x08;
    const HAS_COLOR: u32 = 0x10;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_current_frame(&self) -> bool {
        self.0 & Self::HAS_CURRENT_FRAME == Self::HAS_CURRENT_FRAME
    }

    pub fn has_sleep(&self) -> bool {
        self.0 & Self::HAS_SLEEP == Self::HAS_SLEEP
    }

    pub fn has_light_levels(&self) -> bool {
        self.0 & Self::HAS_LIGHT_LEVELS == Self::HAS_LIGHT_LEVELS
    }

    pub fn skip_frames(&self) -> bool {
        self.0 & Self::SKIP_FRAMES == Self::SKIP_FRAMES
    }

    pub fn has_color(&self) -> bool {
        self.0 & Self::HAS_COLOR == Self::HAS_COLOR
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Debug for LightDefFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LightDefFlags [{:b}]", self.0)
    }
}

impl From<LightDefFlags> for u32 {
    fn from(value: LightDefFlags) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1728-0x1b.frag")[..];
        let frag = LightDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-29288));
        assert_eq!(frag.flags.0, 0x04);
        assert_eq!(frag.frame_count, 1);
        assert_eq!(frag.current_frame, None);
        assert_eq!(frag.sleep, None);
        assert_eq!(frag.light_levels, Some(vec![1.0]));
        assert_eq!(frag.colors, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1728-0x1b.frag")[..];
        let frag = LightDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
