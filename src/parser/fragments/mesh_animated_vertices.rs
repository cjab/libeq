use std::any::Any;

use super::{Fragment, FragmentType, StringReference};

use nom::multi::count;
use nom::number::complete::{le_i16, le_u16, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
/// This fragment contains sets of vertex values to be substituted for the
/// vertex values in a 0x36 Mesh fragment if that mesh is animated. For example,
/// if a mesh has 50 vertices then this fragment will have one or more sets of
/// 50 vertices, one set for each animation frame. The vertex values in this
/// fragment will then be used instead of the vertex values in the 0x36 Mesh
/// fragment as the client cycles through the animation frames.
///
/// **Type ID:** 0x37
pub struct MeshAnimatedVerticesFragment {
    pub name_reference: StringReference,

    /// _Unknown_ - Usually contains 0.
    pub flags: u32,

    /// Should be equal to the number of vertices in the mesh,
    /// as contained in its 0x36 Mesh fragment.
    pub vertex_count: u16,

    /// The number of animation frames.
    pub frame_count: u16,

    /// _Unknown_ - Usually contains 100.
    pub param1: u16,

    /// _Unknown_ - Usually contains 0.
    pub param2: u16,

    /// This works in exactly the same way as the Scale field in the 0x36 Mesh
    /// fragment. By dividing the vertex values by (1 shl Scale), real vertex
    /// values are created.
    pub scale: u16,

    /// There are `frame_count` frames containing `vertex_count` vertices each.
    /// Components of the vertex positions, multiplied by (1 shl Scale).
    pub frames: Vec<Vec<(i16, i16, i16)>>,

    /// _Unknown_ - Usually contains 0.
    pub size6: u16,
}

impl FragmentType for MeshAnimatedVerticesFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x37;
    const TYPE_NAME: &'static str = "MeshAnimatedVertices";

    fn parse(input: &[u8]) -> IResult<&[u8], MeshAnimatedVerticesFragment> {
        let (i, (name_reference, flags, vertex_count, frame_count, param1, param2, scale)) =
            tuple((
                StringReference::parse,
                le_u32,
                le_u16,
                le_u16,
                le_u16,
                le_u16,
                le_u16,
            ))(input)?;
        let (remaining, (frames, size6)) = tuple((
            count(
                count(tuple((le_i16, le_i16, le_i16)), vertex_count as usize),
                frame_count as usize,
            ),
            le_u16,
        ))(i)?;

        Ok((
            remaining,
            MeshAnimatedVerticesFragment {
                name_reference,
                flags,
                vertex_count,
                frame_count,
                param1,
                param2,
                scale,
                frames,
                size6,
            },
        ))
    }
}

impl Fragment for MeshAnimatedVerticesFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.name_reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.vertex_count.to_le_bytes()[..],
            &self.frame_count.to_le_bytes()[..],
            &self.param1.to_le_bytes()[..],
            &self.param2.to_le_bytes()[..],
            &self.scale.to_le_bytes()[..],
            &self
                .frames
                .iter()
                .flat_map(|f| {
                    f.iter().flat_map(|x| {
                        [x.0.to_le_bytes(), x.1.to_le_bytes(), x.2.to_le_bytes()].concat()
                    })
                })
                .collect::<Vec<_>>()[..],
            &self.size6.to_le_bytes()[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark_obj/0631-0x37.frag")[..];
        let frag = MeshAnimatedVerticesFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-5038));
        assert_eq!(frag.flags, 0x0);
        assert_eq!(frag.vertex_count, 104);
        assert_eq!(frag.frame_count, 15);
        assert_eq!(frag.param1, 67);
        assert_eq!(frag.param2, 0);
        assert_eq!(frag.scale, 10);
        assert_eq!(frag.frames.len(), 15);
        assert_eq!(frag.frames[0].len(), 104);
        assert_eq!(frag.frames[0][0], (455, 2180, 11078));
        assert_eq!(frag.size6, 0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark_obj/0631-0x37.frag")[..];
        let frag = MeshAnimatedVerticesFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
