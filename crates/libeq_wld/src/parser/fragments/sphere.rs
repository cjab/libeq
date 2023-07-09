use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::number::complete::le_f32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// SPHERE fragment
///
/// **Type ID:** 0x16
pub struct Sphere {
    pub name_reference: StringReference,

    pub radius: f32,
}

impl FragmentParser for Sphere {
    type T = Self;

    const TYPE_ID: u32 = 0x16;
    const TYPE_NAME: &'static str = "Sphere";

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (remaining, radius) = le_f32(i)?;

        Ok((
            remaining,
            Self {
                name_reference,
                radius,
            },
        ))
    }
}

impl Fragment for Sphere {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4640-0x16.frag")[..];
        let frag = Sphere::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4640-0x16.frag")[..];
        let frag = Sphere::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
