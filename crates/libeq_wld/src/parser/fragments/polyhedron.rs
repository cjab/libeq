use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, PolyhedronDef, StringReference, WResult};

use nom::number::complete::{le_f32, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [PolyhedronDef].
///
/// **Type ID:** 0x18
pub struct Polyhedron {
    pub name_reference: StringReference,

    /// The [PolyhedronDef] reference.
    pub reference: FragmentRef<PolyhedronDef>,

    /// _Unknown_
    /// * bit 0 - If set `params1` exists.
    pub flags: PolyhedronFlags,

    /// _Unknown_
    pub scale_factor: Option<f32>,
}

impl FragmentParser for Polyhedron {
    type T = Self;

    const TYPE_ID: u32 = 0x18;
    const TYPE_NAME: &'static str = "Polyhedron";

    fn parse(input: &[u8]) -> WResult<'_, Polyhedron> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, reference) = FragmentRef::parse(i)?;
        let (i, flags) = PolyhedronFlags::parse(i)?;
        let (i, scale_factor) = if flags.has_scale_factor() {
            le_f32(i).map(|(i, s)| (i, Some(s)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            Self {
                name_reference,
                reference,
                flags,
                scale_factor,
            },
        ))
    }
}

impl Fragment for Polyhedron {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self
                .scale_factor
                .map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
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
pub struct PolyhedronFlags(u32);

impl PolyhedronFlags {
    const HAS_SCALE_FACTOR: u32 = 0x01;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_scale_factor(&self) -> bool {
        self.0 & Self::HAS_SCALE_FACTOR == Self::HAS_SCALE_FACTOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/1418-0x18.frag")[..];
        let frag = Polyhedron::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x058a));
        assert_eq!(frag.flags, PolyhedronFlags(0));
        assert_eq!(frag.scale_factor, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/1418-0x18.frag")[..];
        let frag = Polyhedron::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
