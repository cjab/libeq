use std::any::Any;

use super::{Fragment, FragmentType, StringHash};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
/// This fragment is poorly understood. It seems to contain 26 parameters, some of which
/// are DWORDS (32-bit integers) and some of which are FLOATS (32-bit floating-point values).
/// Until more is known, they are here described as Params[0..25] and their known values
/// are documented.
///
/// In main zone files, the name of this fragment always seems to be CAMERA_DUMMY.
///
/// **Type ID:** 0x08
pub struct CameraFragment {
    /// _Unknown_ - Usually 0
    pub params0: u32,

    /// _Unknown_ - Usually 0
    pub params1: u32,

    /// _Unknown_ - Usually 1
    pub params2: f32,

    /// _Unknown_ - Usually 0
    pub params3: u32,

    /// _Unknown_ - Usually 0
    pub params4: u32,

    /// _Unknown_ - Usually -1.0
    pub params5: f32,

    /// _Unknown_ - Usually 1.0
    pub params6: f32,

    /// _Unknown_ - Usually 0
    pub params7: u32,

    /// _Unknown_ - Usually 1.0
    pub params8: f32,

    /// _Unknown_ - Usually 1.0
    pub params9: f32,

    /// _Unknown_ - Usually 0
    pub params10: u32,

    /// _Unknown_ - Usually 1.0
    pub params11: f32,

    /// _Unknown_ - Usually -1.0
    pub params12: f32,

    /// _Unknown_ - Usually 0
    pub params13: u32,

    /// _Unknown_ - Usually -1.0
    pub params14: f32,

    /// _Unknown_ - Usually -1.0
    pub params15: f32,

    /// _Unknown_ - Usually 4
    pub params16: u32,

    /// _Unknown_ - Usually 0
    pub params17: u32,

    /// _Unknown_ - Usually 0
    pub params18: u32,

    /// _Unknown_ - Usually 0
    pub params19: u32,

    /// _Unknown_ - Usually 1
    pub params20: u32,

    /// _Unknown_ - Usually 2
    pub params21: u32,

    /// _Unknown_ - Usually 3
    pub params22: u32,

    /// _Unknown_ - Usually 0
    pub params23: u32,

    /// _Unknown_ - Usually 1
    pub params24: u32,

    /// _Unknown_ - Usually 11
    pub params25: u32,
}

impl FragmentType for CameraFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x08;

    fn parse(input: &[u8]) -> IResult<&[u8], CameraFragment> {
        let (
            i,
            (
                params0,
                params1,
                params2,
                params3,
                params4,
                params5,
                params6,
                params7,
                params8,
                params9,
            ),
        ) = tuple((
            le_u32, le_u32, le_f32, le_u32, le_u32, le_f32, le_f32, le_u32, le_f32, le_f32,
        ))(input)?;

        let (
            remaining,
            (
                params10,
                params11,
                params12,
                params13,
                params14,
                params15,
                params16,
                params17,
                params18,
                params19,
                params20,
                params21,
                params22,
                params23,
                params24,
                params25,
            ),
        ) = tuple((
            le_u32, le_f32, le_f32, le_u32, le_f32, le_f32, le_u32, le_u32, le_u32, le_u32, le_u32,
            le_u32, le_u32, le_u32, le_u32, le_u32,
        ))(i)?;

        Ok((
            remaining,
            CameraFragment {
                params0,
                params1,
                params2,
                params3,
                params4,
                params5,
                params6,
                params7,
                params8,
                params9,
                params10,
                params11,
                params12,
                params13,
                params14,
                params15,
                params16,
                params17,
                params18,
                params19,
                params20,
                params21,
                params22,
                params23,
                params24,
                params25,
            },
        ))
    }
}

impl Fragment for CameraFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.params0.to_le_bytes()[..],
            &self.params1.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self.params3.to_le_bytes()[..],
            &self.params4.to_le_bytes()[..],
            &self.params5.to_le_bytes()[..],
            &self.params6.to_le_bytes()[..],
            &self.params7.to_le_bytes()[..],
            &self.params8.to_le_bytes()[..],
            &self.params9.to_le_bytes()[..],
            &self.params10.to_le_bytes()[..],
            &self.params11.to_le_bytes()[..],
            &self.params12.to_le_bytes()[..],
            &self.params13.to_le_bytes()[..],
            &self.params14.to_le_bytes()[..],
            &self.params15.to_le_bytes()[..],
            &self.params16.to_le_bytes()[..],
            &self.params17.to_le_bytes()[..],
            &self.params18.to_le_bytes()[..],
            &self.params19.to_le_bytes()[..],
            &self.params20.to_le_bytes()[..],
            &self.params21.to_le_bytes()[..],
            &self.params22.to_le_bytes()[..],
            &self.params23.to_le_bytes()[..],
            &self.params24.to_le_bytes()[..],
            &self.params25.to_le_bytes()[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self, string_hash: &StringHash) -> String {
        String::new()
    }
}
