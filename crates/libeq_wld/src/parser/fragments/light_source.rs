use std::any::Any;

use super::{Fragment, FragmentParser, StringReference};

use nom::number::complete::{le_f32, le_u32, le_u8};
use nom::sequence::tuple;
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// **Type ID:** 0x1b
pub struct LightSourceFragment {
    pub name_reference: StringReference,

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
    pub flags: u32,

    /// _Unknown_ - Usually contains 1
    pub params2: u32,

    /// _Unknown_ - Usually contains 1
    pub params3a: Option<f32>,

    /// _Unknown_ - Usually contains 200 (attenuation?).
    pub params3b: Option<u32>,

    /// _Unknown_ - Usually contains 1.
    pub params4: Option<u8>,

    /// Red component, scaled from 0 (no red component) to 1 (100% red).
    pub red: Option<u8>,

    /// Green component, scaled from 0 (no green component) to 1 (100% green).
    pub green: Option<u8>,

    /// Blue component, scaled from 0 (no blue component) to 1 (100% blue).
    pub blue: Option<u8>,
}

impl FragmentParser for LightSourceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x1b;
    const TYPE_NAME: &'static str = "LightSource";

    fn parse(input: &[u8]) -> IResult<&[u8], LightSourceFragment> {
        let (i, (name_reference, flags, params2)) =
            tuple((StringReference::parse, le_u32, le_u32))(input)?;

        let (i, params3a) = if flags & 0x10 == 0x00 {
            le_f32(i).map(|(i, params3a)| (i, Some(params3a)))?
        } else {
            (i, None)
        };

        let (i, params3b) = if flags & 0x18 == 0x18 {
            le_u32(i).map(|(i, params3b)| (i, Some(params3b)))?
        } else {
            (i, None)
        };

        let (remaining, (params4, red, green, blue)) = if flags & 0x10 == 0x10 {
            tuple((le_u8, le_u8, le_u8, le_u8))(i)
                .map(|(i, (p4, r, g, b))| (i, (Some(p4), Some(r), Some(g), Some(b))))?
        } else {
            (i, (None, None, None, None))
        };

        Ok((
            remaining,
            LightSourceFragment {
                name_reference,
                flags,
                params2,
                params3a,
                params3b,
                params4,
                red,
                green,
                blue,
            },
        ))
    }
}

impl Fragment for LightSourceFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self.params3a.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.params3b.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.params4.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.red.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.green.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
            &self.blue.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1728-0x1b.frag")[..];
        let frag = LightSourceFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-29288));
        assert_eq!(frag.flags, 0x04);
        assert_eq!(frag.params2, 1);
        assert_eq!(frag.params3a, Some(1.0));
        assert_eq!(frag.params3b, None);
        assert_eq!(frag.params4, None);
        assert_eq!(frag.red, None);
        assert_eq!(frag.green, None);
        assert_eq!(frag.blue, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1728-0x1b.frag")[..];
        let frag = LightSourceFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
