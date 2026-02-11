use super::{Fragment, FragmentParser, StringReference, WResult};
use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use std::any::Any;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// WORLDVERTICES
///
/// **Type ID:** 0x2c
pub struct WorldVertices {
    pub name_reference: StringReference,

    /// NUMVERTICES %d
    pub num_vertices: u32,

    /// XYZ %f, %f, %f
    pub vertices: Vec<(f32, f32, f32)>,
}

impl FragmentParser for WorldVertices {
    type T = Self;

    const TYPE_ID: u32 = 0x2c;
    const TYPE_NAME: &'static str = "WorldVertices";

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let name_reference = StringReference::new(0);
        let (i, num_vertices) = le_u32(input)?;
        let (i, vertices) = count((le_f32, le_f32, le_f32), num_vertices as usize).parse(i)?;

        Ok((
            i,
            Self {
                name_reference,
                num_vertices,
                vertices,
            },
        ))
    }
}

impl Fragment for WorldVertices {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.num_vertices.to_le_bytes()[..],
            &self
                .vertices
                .iter()
                .flat_map(|v| [v.0.to_le_bytes(), v.1.to_le_bytes(), v.2.to_le_bytes()].concat())
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

    #[test]
    fn it_parses() {
        let data =
            &include_bytes!("../../../fixtures/fragments/tanarus-thecity/0001-0x2c.frag")[..];
        let (remaining, frag) = WorldVertices::parse(data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.num_vertices, 9702);
        assert_eq!(frag.vertices[0], (-960.00006, -32.0, 48.0));
        assert_eq!(frag.vertices[9701], (960.00006, 32.0, 31.999992));
        assert_eq!(remaining, vec![]);
    }

    #[test]
    fn it_serializes() {
        let data =
            &include_bytes!("../../../fixtures/fragments/tanarus-thecity/0001-0x2c.frag")[..];
        let frag = WorldVertices::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
