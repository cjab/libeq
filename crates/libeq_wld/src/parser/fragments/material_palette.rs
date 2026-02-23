use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, MaterialDef, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::le_u32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
///
/// **Type ID:** 0x31
pub struct MaterialPalette {
    pub name_reference: StringReference,

    /// _Unknown_ - Must contain 0.
    pub flags: u32,

    /// The number of fragment references this fragment contains.
    pub size1: u32,

    /// `size1` references to [MaterialDef] fragments.
    pub fragments: Vec<FragmentRef<MaterialDef>>,
}

impl FragmentParser for MaterialPalette {
    type T = Self;

    const TYPE_ID: u32 = 0x31;
    const TYPE_NAME: &'static str = "MaterialPalette";

    fn parse(input: &[u8]) -> WResult<'_, MaterialPalette> {
        let (i, (name_reference, flags, size1)) =
            (StringReference::parse, le_u32, le_u32).parse(input)?;
        let (remaining, fragments) = count(FragmentRef::parse, size1 as usize).parse(i)?;
        Ok((
            remaining,
            MaterialPalette {
                name_reference,
                flags,
                size1,
                fragments,
            },
        ))
    }
}

impl Fragment for MaterialPalette {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.size1.to_le_bytes()[..],
            &self
                .fragments
                .iter()
                .flat_map(|f| f.to_bytes())
                .collect::<Vec<_>>()[..],
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

    fn fixture() -> MaterialPalette {
        MaterialPalette {
            name_reference: StringReference::new(-100),
            flags: 0,
            size1: 3,
            fragments: vec![
                FragmentRef::new(5),
                FragmentRef::new(10),
                FragmentRef::new(15),
            ],
        }
    }

    #[test]
    fn it_parses() {
        let data = fixture().to_bytes();
        let frag = MaterialPalette::parse(&data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-100));
        assert_eq!(frag.flags, 0);
        assert_eq!(frag.size1, 3);
        assert_eq!(frag.fragments.len(), 3);
        assert_eq!(frag.fragments[0], FragmentRef::new(5));
        assert_eq!(frag.fragments[1], FragmentRef::new(10));
        assert_eq!(frag.fragments[2], FragmentRef::new(15));
    }

    #[test]
    fn it_serializes() {
        let data = fixture().to_bytes();
        let frag = MaterialPalette::parse(&data).unwrap().1;

        assert_eq!(frag.to_bytes(), data);
    }
}
