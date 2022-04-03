use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, StringReference};

use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
/// A reference to a [TwoDimensionalObjectReferenceFragment].
///
/// **Type ID:** 0x07
pub struct TwoDimensionalObjectReferenceFragment {
    pub name_reference: StringReference,

    /// The [TwoDimensionalObjectFragment] reference.
    pub reference: FragmentRef<TwoDimensionalObjectReferenceFragment>,

    /// _Unknown_ Seems to always contain 0.
    pub flags: u32,
}

impl FragmentType for TwoDimensionalObjectReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x07;

    fn parse(input: &[u8]) -> IResult<&[u8], TwoDimensionalObjectReferenceFragment> {
        let (remaining, (name_reference, reference, flags)) =
            tuple((StringReference::parse, fragment_ref, le_u32))(input)?;
        Ok((
            remaining,
            TwoDimensionalObjectReferenceFragment {
                name_reference,
                reference,
                flags,
            },
        ))
    }
}

impl Fragment for TwoDimensionalObjectReferenceFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gequip/2224-0x07.frag")[..];
        let frag = TwoDimensionalObjectReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.reference, FragmentRef::new(0x07f0));
        assert_eq!(frag.flags, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/2224-0x07.frag")[..];
        let frag = TwoDimensionalObjectReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
