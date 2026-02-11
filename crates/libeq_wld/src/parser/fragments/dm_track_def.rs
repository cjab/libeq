use super::{Fragment, FragmentParser, StringReference, WResult};
use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use std::any::Any;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// DMTRACKDEF
///
/// An older version of the 0x37 fragment, which describes the animation of individual vertices in a DMSPRITEDEF mesh.
///
/// **Type ID:** 0x2e
pub struct DmTrackDef {
    pub name_reference: StringReference,

    pub flags: u32,
    pub vertex_count: u32,
    pub frame_count: u32,
    pub sleep: u32,
    pub param1: u32,
    pub frames: Vec<Vec<(f32, f32, f32)>>,
}

impl FragmentParser for DmTrackDef {
    type T = Self;

    const TYPE_ID: u32 = 0x2e;
    const TYPE_NAME: &'static str = "DmTrackDef";

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, name_reference) = StringReference::parse(input)?;

        let (i, (flags, vertex_count, frame_count, sleep, param1)) =
            (le_u32, le_u32, le_u32, le_u32, le_u32).parse(i)?;

        let (i, frames) = count(
            count((le_f32, le_f32, le_f32), vertex_count as usize),
            frame_count as usize,
        )
        .parse(i)?;

        Ok((
            i,
            Self {
                name_reference,
                flags,
                vertex_count,
                frame_count,
                sleep,
                param1,
                frames,
            },
        ))
    }
}

impl Fragment for DmTrackDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_be_bytes()[..],
            &self.vertex_count.to_le_bytes()[..],
            &self.frame_count.to_le_bytes()[..],
            &self.sleep.to_le_bytes()[..],
            &self.param1.to_le_bytes()[..],
            &self
                .frames
                .iter()
                .flat_map(|f| {
                    f.iter().flat_map(|x| {
                        [x.0.to_le_bytes(), x.1.to_le_bytes(), x.2.to_le_bytes()].concat()
                    })
                })
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
