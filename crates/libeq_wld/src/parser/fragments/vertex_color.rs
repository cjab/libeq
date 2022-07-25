use std::any::Any;

use super::{Fragment, FragmentParser, StringReference};

use nom::multi::count;
use nom::number::complete::le_u32;
use nom::sequence::tuple;
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// **Type ID:** 0x32
pub struct VertexColorFragment {
    pub name_reference: StringReference,

    /// _Unknown_ - Usually contains 1.
    pub data1: u32,

    /// The number of color values in the `vertex_colors` list. It should be equal
    /// to the number of vertices in the placeable object, as contained in its 0x36
    /// [MeshFragment].
    pub vertex_color_count: u32,

    /// _Unknown_ - Usually contains 1.
    pub data2: u32,

    /// _Unknown_ - Usually contains 200.
    pub data3: u32,

    /// _Unknown_ - Usually contains 0.
    pub data4: u32,

    /// This contains an RGBA color value for each vertex in the placeable object. It
    /// specifies the additional color to be applied to the vertex, as if that vertex
    /// has been illuminated by a nearby light source. The A value isn’t fully understood;
    /// I believe it represents an alpha as applied to the texture, such that 0 makes the
    /// polygon a pure color and 0xFF either illuminates an unaltered texture or mutes the
    /// illumination completely. That is, it’s either a blending value or an alpha value.
    /// Further experimentation is required. 0xD9 seems to be a good (typical) A value for
    /// most illuminated vertices.
    ///
    /// This field works in exactly the same way as it does in the 0x36 [MeshFragment].
    pub vertex_colors: Vec<u32>,
}

impl FragmentParser for VertexColorFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x32;
    const TYPE_NAME: &'static str = "VertexColor";

    fn parse(input: &[u8]) -> IResult<&[u8], VertexColorFragment> {
        let (i, (name_reference, data1, vertex_color_count, data2, data3, data4)) =
            tuple((
                StringReference::parse,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
            ))(input)?;
        let (remaining, vertex_colors) = count(le_u32, vertex_color_count as usize)(i)?;

        Ok((
            remaining,
            VertexColorFragment {
                name_reference,
                data1,
                vertex_color_count,
                data2,
                data3,
                data4,
                vertex_colors,
            },
        ))
    }
}

impl Fragment for VertexColorFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.data1.to_le_bytes()[..],
            &self.vertex_color_count.to_le_bytes()[..],
            &self.data2.to_le_bytes()[..],
            &self.data3.to_le_bytes()[..],
            &self.data4.to_le_bytes()[..],
            &self
                .vertex_colors
                .iter()
                .flat_map(|v| v.to_le_bytes())
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
        let data = &include_bytes!("../../../fixtures/fragments/objects/0000-0x32.frag")[..];
        let frag = VertexColorFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-1));
        assert_eq!(frag.data1, 1);
        assert_eq!(frag.vertex_color_count, 176);
        assert_eq!(frag.data2, 1);
        assert_eq!(frag.data3, 200);
        assert_eq!(frag.data4, 0);
        assert_eq!(frag.vertex_colors.len(), 176);
        assert_eq!(frag.vertex_colors[0], 3254779903);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/objects/0000-0x32.frag")[..];
        let frag = VertexColorFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
