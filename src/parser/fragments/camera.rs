use std::any::Any;

use super::{Fragment, FragmentParser, StringReference};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
/// This fragment is poorly understood. It seems to contain 26 parameters, some of which
/// are DWORDS (32-bit integers) and some of which are FLOATS (32-bit floating-point values).
/// Until more is known, they are here described as Params[0..25] and their known values
/// are documented.
///
/// In main zone files, the name of this fragment always seems to be CAMERA_DUMMY.
///
/// **Type ID:** 0x08
pub struct CameraFragment {
    pub name_reference: StringReference,

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

impl FragmentParser for CameraFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x08;
    const TYPE_NAME: &'static str = "Camera";

    fn parse(input: &[u8]) -> IResult<&[u8], CameraFragment> {
        let (
            i,
            (
                name_reference,
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
            StringReference::parse,
            le_u32,
            le_u32,
            le_f32,
            le_u32,
            le_u32,
            le_f32,
            le_f32,
            le_u32,
            le_f32,
            le_f32,
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
                name_reference,
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
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
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

    fn name_ref(&self) -> &StringReference {
        &self.name_reference
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1729-0x08.frag")[..];
        let frag = CameraFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-29305));
        assert_eq!(frag.params0, 0);
        assert_eq!(frag.params1, 4);
        assert_eq!(frag.params2, 1e-45);
        assert_eq!(frag.params3, 0);
        assert_eq!(frag.params4, 0);
        assert_eq!(frag.params5, -1.0);
        assert_eq!(frag.params6, 1.0);
        assert_eq!(frag.params7, 0);
        assert_eq!(frag.params8, 1.0);
        assert_eq!(frag.params9, 1.0);
        assert_eq!(frag.params10, 0);
        assert_eq!(frag.params11, 1.0);
        assert_eq!(frag.params12, -1.0);
        assert_eq!(frag.params13, 0);
        assert_eq!(frag.params14, -1.0);
        assert_eq!(frag.params15, -1.0);
        assert_eq!(frag.params16, 4);
        assert_eq!(frag.params17, 0);
        assert_eq!(frag.params18, 0);
        assert_eq!(frag.params19, 0);
        assert_eq!(frag.params20, 1);
        assert_eq!(frag.params21, 2);
        assert_eq!(frag.params22, 3);
        assert_eq!(frag.params23, 0);
        assert_eq!(frag.params24, 1);
        assert_eq!(frag.params25, 11);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/1729-0x08.frag")[..];
        let frag = CameraFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
