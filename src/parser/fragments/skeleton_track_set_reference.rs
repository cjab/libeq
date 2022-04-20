use std::any::Any;

use super::{Fragment, FragmentRef, FragmentParser, SkeletonTrackSetFragment, StringReference};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
/// A reference to a [SkeletonTrackSetFragment].
///
/// **Type ID:** 0x11
pub struct SkeletonTrackSetReferenceFragment {
    pub name_reference: StringReference,

    /// The [SkeletonTrackSetFragment] reference.
    pub reference: FragmentRef<SkeletonTrackSetFragment>,

    /// _Unknown_ Seems to always contain 0.
    pub params1: u32,
}

impl FragmentParser for SkeletonTrackSetReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x11;
    const TYPE_NAME: &'static str = "SkeletonTrackSetReference";

    fn parse(input: &[u8]) -> IResult<&[u8], SkeletonTrackSetReferenceFragment> {
        let (remaining, (name_reference, reference, params1)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            SkeletonTrackSetReferenceFragment {
                name_reference,
                reference,
                params1,
            },
        ))
    }
}

impl Fragment for SkeletonTrackSetReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.reference.serialize()[..],
            &self.params1.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gequip/2228-0x11.frag")[..];
        let frag = SkeletonTrackSetReferenceFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x0e));
        assert_eq!(frag.params1, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/2228-0x11.frag")[..];
        let frag = SkeletonTrackSetReferenceFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
