use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::Parser;
use nom::combinator::opt;
use nom::multi::count;
use nom::number::complete::{le_i16, le_u16, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// This fragment contains sets of vertex values to be substituted for the
/// vertex values in a 0x36 Mesh fragment if that mesh is animated. For example,
/// if a mesh has 50 vertices then this fragment will have one or more sets of
/// 50 vertices, one set for each animation frame. The vertex values in this
/// fragment will then be used instead of the vertex values in the 0x36 Mesh
/// fragment as the client cycles through the animation frames.
///
/// **Type ID:** 0x37
pub struct DmTrackDef2 {
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

    /// The two bytes after the frames, when the fragment has them. They pad
    /// 18 + 6 × `vertex_count` × `frame_count` bytes out to a multiple of 4,
    /// so a fragment whose frames already end on a 4-byte boundary has none.
    pub size6: Option<u16>,
}

impl FragmentParser for DmTrackDef2 {
    type T = Self;

    const TYPE_ID: u32 = 0x37;
    const TYPE_NAME: &'static str = "DmTrackDef2";

    fn parse(input: &[u8]) -> WResult<'_, DmTrackDef2> {
        let (i, (name_reference, flags, vertex_count, frame_count, param1, param2, scale)) = (
            StringReference::parse,
            le_u32,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
        )
            .parse(input)?;
        let (remaining, (frames, size6)) = (
            count(
                count((le_i16, le_i16, le_i16), vertex_count as usize),
                frame_count as usize,
            ),
            opt(le_u16),
        )
            .parse(i)?;

        Ok((
            remaining,
            DmTrackDef2 {
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

impl Fragment for DmTrackDef2 {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
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
            &self.size6.map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
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

    fn fixture() -> DmTrackDef2 {
        DmTrackDef2 {
            name_reference: StringReference::new(-5038),
            flags: 0x0,
            vertex_count: 3,
            frame_count: 2,
            param1: 67,
            param2: 0,
            scale: 10,
            frames: vec![
                vec![(455, 2180, 11078), (100, 200, 300), (400, 500, 600)],
                vec![(460, 2185, 11083), (105, 205, 305), (405, 505, 605)],
            ],
            size6: Some(0),
        }
    }

    #[test]
    fn it_parses() {
        let data = &fixture().to_bytes()[..];
        let frag = DmTrackDef2::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-5038));
        assert_eq!(frag.flags, 0x0);
        assert_eq!(frag.vertex_count, 3);
        assert_eq!(frag.frame_count, 2);
        assert_eq!(frag.param1, 67);
        assert_eq!(frag.param2, 0);
        assert_eq!(frag.scale, 10);
        assert_eq!(frag.frames.len(), 2);
        assert_eq!(frag.frames[0].len(), 3);
        assert_eq!(frag.frames[0][0], (455, 2180, 11078));
        assert_eq!(frag.size6, Some(0));
    }

    /// A fragment as the format lays it out: the 18-byte head, then the frames.
    fn bytes(vertex_count: u16, frame_count: u16) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend((-7i32).to_le_bytes()); // name
        data.extend(0u32.to_le_bytes()); // flags
        for word in [vertex_count, frame_count, 100, 0, 9] {
            data.extend(word.to_le_bytes());
        }
        for i in 0..i16::try_from(vertex_count * frame_count).unwrap() {
            for axis in [i, -i, 2 * i] {
                data.extend(axis.to_le_bytes());
            }
        }
        data
    }

    #[test]
    fn it_parses_frames_that_end_on_a_word_boundary() {
        // 18 + 6 × 1 × 3 = 36 bytes: nothing follows the frames.
        let data = bytes(1, 3);
        assert_eq!(data.len(), 36);
        let (remaining, frag) = DmTrackDef2::parse(&data).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(
            frag.frames,
            vec![vec![(0, 0, 0)], vec![(1, -1, 2)], vec![(2, -2, 4)]]
        );
        assert_eq!(frag.size6, None);
        assert_eq!(frag.to_bytes(), data);
    }

    #[test]
    fn it_parses_frames_followed_by_padding() {
        // 18 + 6 × 1 × 2 = 30 bytes, padded with two more.
        let mut data = bytes(1, 2);
        data.extend([0, 0]);
        let (remaining, frag) = DmTrackDef2::parse(&data).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(frag.frames, vec![vec![(0, 0, 0)], vec![(1, -1, 2)]]);
        assert_eq!(frag.size6, Some(0));
        assert_eq!(frag.to_bytes(), data);
    }

    #[test]
    fn it_rejects_frames_cut_short() {
        let mut data = bytes(1, 3);
        data.truncate(data.len() - 4);

        assert!(DmTrackDef2::parse(&data).is_err());
    }

    #[test]
    fn it_serializes() {
        let frag = fixture();
        let data = frag.to_bytes();
        let parsed = DmTrackDef2::parse(&data).unwrap().1;

        assert_eq!(parsed.to_bytes(), data);
    }
}
