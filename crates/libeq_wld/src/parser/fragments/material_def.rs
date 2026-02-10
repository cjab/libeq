use std::any::Any;

use super::common::RenderMethod;
use super::{Fragment, FragmentParser, FragmentRef, SimpleSprite, StringReference, WResult};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
///
/// **Type ID:** 0x30
pub struct MaterialDef {
    pub name_reference: StringReference,

    /// Most flags are _unknown_, however:
    /// * bit 1 - If set then the `pair` field exists. This is usually set.
    pub flags: u32,

    /// Most flags are _unknown_, however:
    // Bit 0 ........ Apparently must be 1 if the texture isn’t transparent.
    // Bit 1 ........ Set to 1 if the texture is masked (e.g. tree leaves).
    // Bit 2 ........ Set to 1 if the texture is semi-transparent but not masked.
    // Bit 3 ........ Set to 1 if the texture is masked and semi-transparent.
    // Bit 4 ........ Set to 1 if the texture is masked but not semi-transparent.
    // Bit 31 ...... Apparently must be 1 if the texture isn’t transparent.
    pub render_method: RenderMethod,

    /// This typically contains 0x004E4E4E but has also bee known to contain 0xB2B2B2.
    /// Could this be an RGB reflectivity value?
    /// RGBPEN %d, %d, %d
    pub rgb_pen: u32,

    /// BRIGHTNESS %f
    pub brightness: f32,

    /// SCALEDAMBIENT %f
    pub scaled_ambient: f32,

    /// A reference to a [SimpleSprite] fragment.
    pub reference: FragmentRef<SimpleSprite>,

    /// _Unknown_ - This only exists if bit 1 of flags is set. Both fields usually contain 0.
    pub pair: Option<(u32, f32)>,
}

impl FragmentParser for MaterialDef {
    type T = Self;

    const TYPE_ID: u32 = 0x30;
    const TYPE_NAME: &'static str = "MaterialDef";

    fn parse(input: &[u8]) -> WResult<'_, MaterialDef> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = le_u32(i)?;
        let (i, render_method) = RenderMethod::parse(i)?;
        let (i, rgb_pen) = le_u32(i)?;
        let (i, brightness) = le_f32(i)?;
        let (i, scaled_ambient) = le_f32(i)?;
        let (i, reference) = FragmentRef::parse(i)?;

        let (i, pair) = if flags & 0x2 == 0x2 {
            tuple((le_u32, le_f32))(i).map(|(rem, p)| (rem, Some(p)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            MaterialDef {
                name_reference,
                flags,
                render_method,
                rgb_pen,
                brightness,
                scaled_ambient,
                reference,
                pair,
            },
        ))
    }
}

impl Fragment for MaterialDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.render_method.into_bytes()[..],
            &self.rgb_pen.to_le_bytes()[..],
            &self.brightness.to_le_bytes()[..],
            &self.scaled_ambient.to_le_bytes()[..],
            &self.reference.into_bytes()[..],
            &self
                .pair
                .map_or(vec![], |p| [p.0.to_le_bytes(), p.1.to_le_bytes()].concat())[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0004-0x30.frag")[..];
        let frag = MaterialDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-22));
        assert_eq!(frag.flags, 0x02);
        assert_eq!(
            <RenderMethod as std::convert::Into<u32>>::into(frag.render_method),
            0x80000001
        );
        assert_eq!(frag.rgb_pen, 0x4e4e4e);
        assert_eq!(frag.brightness, 0.0);
        assert_eq!(frag.scaled_ambient, 0.75);
        assert_eq!(frag.reference, FragmentRef::new(4));
        assert_eq!(frag.pair, Some((0, 0.0)));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0004-0x30.frag")[..];
        let frag = MaterialDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
