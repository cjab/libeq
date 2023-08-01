use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, SphereListDef, StringReference, WResult};

use nom::number::complete::le_u32;
use nom::sequence::tuple;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// A reference to a [SphereListDef].
///
/// **Type ID:** 0x1a
pub struct SphereList {
    pub name_reference: StringReference,

    /// The [SphereListDef] reference.
    pub reference: FragmentRef<SphereListDef>,

    /// _Unknown_.
    pub params1: u32,
}

impl FragmentParser for SphereList {
    type T = Self;

    const TYPE_ID: u32 = 0x1a;
    const TYPE_NAME: &'static str = "SphereList";

    fn parse(input: &[u8]) -> WResult<SphereList> {
        let (remaining, (name_reference, reference, params1)) =
            tuple((StringReference::parse, FragmentRef::parse, le_u32))(input)?;
        Ok((
            remaining,
            SphereList {
                name_reference,
                reference,
                params1,
            },
        ))
    }
}

impl Fragment for SphereList {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
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

    fn type_id(&self) -> u32 {
        Self::TYPE_ID
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_a_known_name_reference() {
        #![allow(overflowing_literals)]
        let data = &include_bytes!("../../../fixtures/fragments/tanarus-equip/2903-0x1a.frag")[..];
        let frag = SphereList::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(0x0));
        assert_eq!(frag.reference, FragmentRef::new(0xb47));
        assert_eq!(frag.params1, 0x0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/tanarus-equip/2903-0x1a.frag")[..];
        let frag = SphereList::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
