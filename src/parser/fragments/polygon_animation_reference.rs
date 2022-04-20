use std::any::Any;

use super::{Fragment, FragmentRef, FragmentParser, PolygonAnimationFragment, StringReference};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
/// A reference to a [PolygonAnimationFragment].
///
/// **Type ID:** 0x18
pub struct PolygonAnimationReferenceFragment {
    pub name_reference: StringReference,

    /// The [PolygonAnimationFragment] reference.
    pub reference: FragmentRef<PolygonAnimationFragment>,

    /// _Unknown_
    /// * bit 0 - If set `params1` exists.
    pub flags: u32,

    /// _Unknown_
    pub params1: Option<f32>,
}

impl FragmentParser for PolygonAnimationReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x18;
    const TYPE_NAME: &'static str = "PolygonAnimationReference";

    fn parse(input: &[u8]) -> IResult<&[u8], PolygonAnimationReferenceFragment> {
        let (i, (name_reference, reference, flags)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;

        let (remaining, params1) = if (flags & 0x1) == 0x1 {
            le_f32(i).map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            PolygonAnimationReferenceFragment {
                name_reference,
                reference,
                flags,
                params1,
            },
        ))
    }
}

impl Fragment for PolygonAnimationReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.params1.map_or(vec![], |p| p.to_le_bytes().to_vec())[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name_ref(&self) -> &StringReference {
        &self.name_reference
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/1418-0x18.frag")[..];
        let frag = PolygonAnimationReferenceFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x058a));
        assert_eq!(frag.flags, 0);
        assert_eq!(frag.params1, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/1418-0x18.frag")[..];
        let frag = PolygonAnimationReferenceFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
