use super::{Fragment, FragmentParser, StringReference, WResult};
use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use std::any::Any;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// RGBDEFORMATIONTRACKDEF
///
/// **Type ID:** 0x2e
pub struct Unknown0x2eFragment {
    pub name_reference: StringReference,

    pub flags: u32,
    // NUMVERTICES %d
    pub vertex_count: u32,
    // NUMFRAMES 1
    pub frame_count: u32,
    // SLEEP 200
    pub sleep: u32,
    // Unknown
    pub param1: u32,
    // RGBDEFORMATIONFRAME
    //   NUMRGBAS %d
    //   RGBA %9.7f, %9.7f, %9.7f, %9.7f
    // ENDRGBDEFORMATIONFRAME
    pub frames: Vec<Vec<(f32, f32, f32)>>,
}

impl FragmentParser for Unknown0x2eFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x2e;
    const TYPE_NAME: &'static str = "Unknown0x2e";

    fn parse(input: &[u8]) -> WResult<Self> {
        let (i, name_reference) = StringReference::parse(input)?;

        let (i, (flags, vertex_count, frame_count, sleep, param1)) =
            tuple((le_u32, le_u32, le_u32, le_u32, le_u32))(i)?;

        let (i, frames) = 
            count(
                count(tuple((le_f32, le_f32, le_f32)), vertex_count as usize),
                frame_count as usize,
            )(i)?;
        
        Ok((
            i,
            Self {
                name_reference,
                flags,
                vertex_count,
                frame_count,
                sleep,
                param1,
                frames
            },
        ))
    }
}

impl Fragment for Unknown0x2eFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [&self.name_reference.into_bytes()[..]].concat()
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
