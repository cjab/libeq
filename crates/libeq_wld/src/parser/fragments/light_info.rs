use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, LightSourceReferenceFragment, StringReference};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [LightSourceReferenceFragment].
///
/// **Type ID:** 0x28
pub struct LightInfoFragment {
    pub name_reference: StringReference,

    /// The [LightSourceReferenceFragment] reference.
    pub reference: FragmentRef<LightSourceReferenceFragment>,

    /// _Unknown_ - Usually contains 256 (0x100).
    pub flags: u32,

    /// X component of the light location.
    pub x: f32,

    /// Y component of the light location.
    pub y: f32,

    /// Z component of the light location.
    pub z: f32,

    /// Contains the light radius.
    pub radius: f32,
}

impl FragmentParser for LightInfoFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x28;
    const TYPE_NAME: &'static str = "LightInfo";

    fn parse(input: &[u8]) -> IResult<&[u8], LightInfoFragment> {
        let (remaining, (name_reference, reference, flags, x, y, z, radius)) = tuple((
            StringReference::parse,
            FragmentRef::parse,
            le_u32,
            le_f32,
            le_f32,
            le_f32,
            le_f32,
        ))(input)?;
        Ok((
            remaining,
            LightInfoFragment {
                name_reference,
                reference,
                flags,
                x,
                y,
                z,
                radius,
            },
        ))
    }
}

impl Fragment for LightInfoFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.x.to_le_bytes()[..],
            &self.y.to_le_bytes()[..],
            &self.z.to_le_bytes()[..],
            &self.radius.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/lights/0002-0x28.frag")[..];
        let frag = LightInfoFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(2));
        assert_eq!(frag.flags, 0x100);
        assert_eq!(frag.x, -1980.6992);
        assert_eq!(frag.y, -2354.9412);
        assert_eq!(frag.z, 31.490416);
        assert_eq!(frag.radius, 300.0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/lights/0002-0x28.frag")[..];
        let frag = LightInfoFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
