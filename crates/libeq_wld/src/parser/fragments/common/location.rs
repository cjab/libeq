use nom::number::complete::{le_f32, le_u32};
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Represents LOCATION within an ACTORDEF and ACTORINST.
pub struct Location {
    /// LOCATION      %d   %f   %f   %f       %d       %d       %d
    /// LOCATION unknown    x    y    z rotate_z rotate_y rotate_x

    /// When used in main zone files, contains the minimum X value of the
    /// entire zone. When used for placeable objects, contains the X value
    /// of the object’s location.
    pub x: f32,

    /// When used in main zone files, contains the minimum Y value of the
    /// entire zone. When used for placeable objects, contains the Y value
    /// of the object’s location.
    pub y: f32,

    /// When used in main zone files, contains the minimum Z value of the
    /// entire zone. When used for placeable objects, contains the Z value
    /// of the object’s location.
    pub z: f32,

    /// When used in main zone files, typically contains 0. When used for
    /// placeable objects, contains a value describing rotation around the Z
    /// axis, scaled as Degrees x (512 / 360).
    pub rotate_z: f32,

    /// When used in main zone files, typically contains 0. When used for
    /// placeable objects, contains a value describing rotation around the Y
    /// axis, scaled as Degrees x (512 / 360).
    pub rotate_y: f32,

    /// When used in main zone files, typically contains 0. When used for
    /// placeable objects, contains a value describing rotation around the X
    /// axis, scaled as Degrees x (512 / 360).
    pub rotate_x: f32,

    pub unknown: u32,
}

impl Location {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, x) = le_f32(input)?;
        let (i, y) = le_f32(i)?;
        let (i, z) = le_f32(i)?;
        let (i, rotate_z) = le_f32(i)?;
        let (i, rotate_y) = le_f32(i)?;
        let (i, rotate_x) = le_f32(i)?;
        let (i, unknown) = le_u32(i)?;

        Ok((
            i,
            Self {
                x,
                y,
                z,
                rotate_x,
                rotate_y,
                rotate_z,
                unknown,
            },
        ))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.x.to_le_bytes()[..],
            &self.y.to_le_bytes()[..],
            &self.z.to_le_bytes()[..],
            &self.rotate_z.to_le_bytes()[..],
            &self.rotate_y.to_le_bytes()[..],
            &self.rotate_x.to_le_bytes()[..],
            &self.unknown.to_le_bytes()[..],
        ]
        .concat()
    }
}
