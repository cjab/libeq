use std::any::Any;

use super::{Fragment, FragmentParser, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// ## Notes
/// This fragment describes how a skeleton piece is shifted or rotated relative to its parent
/// piece. The overall skeleton is contained in a 0x10 Skeleton Track Set fragment and is
/// structured as a hierarchical tree (see that fragment for information on how skeletons
/// are structured). The 0x12 fragment contains information on how that particular skeleton
/// piece is rotated and/or shifted relative to its parent piece.
///
/// Rotation and shifting information is contained as a series of fractions. The fragment
/// contains one denominator value for rotation and another for translation (X, Y, Z, shift).
/// It contains one numerator each for X, Y, Z rotation and shift, for a total of eight values.
/// For rotation, the resulting value should be multiplied by Pi / 2 radians (i.e. 1 corresponds
/// to 90 degrees, 2 corresponds to 180 degrees, etc.).
///
/// ## Fields
/// For rendering polygons, the X, Y, Z rotation and shift information in this fragment should
/// be taken into account by adding them to the rotation and shift values passed from the parent
/// piece (that is, rotation and shift are cumulative). However, before adding the shift values,
/// the X, Y, and Z shift values should first be rotated according to the parent’s rotation values.
/// The rotation values in this fragment represent the orientation of this piece relative to the
/// parent so calculating its starting position should not take its own rotation into account.
///
/// Software rendering a skeleton piece should perform the following steps in this order:
///   * Calculate the X, Y, and Z shift values from this fragment
///   * Rotate the shift values according to the rotation values from the parent piece
///   * Add the shift values to the shift values from the parent piece
///   * Calculate the X, Y, and Z rotation values from this fragment
///   * Add the rotation values to the rotation values from the parent piece
///   * Adjust the vertices for this piece by rotating them using the new rotation values and then
///     shifting them by the new shift values (or save the rotation and shift values for this piece
///     to be looked up later on when rendering)
///   * Process the next piece in the tree with the new rotation and shift values
///   * When all pieces have been processed, render all meshes in the model, using either the
///     adjusted vertex values (more efficient) or looking up the corresponding piece for each
///     vertex and adjusting the vertex values according to the adjusted rotation and shift values
///     calculated above (less efficient).
///
/// **Type ID:** 0x12
pub struct TrackDef {
    pub name_reference: StringReference,

    /// Most flags are _unknown_.
    /// * bit 3 - If set then the fragment uses `legacy_frame_transforms`
    pub flags: u32,

    /// The number of `FrameTransform` and `LegacyFrameTransform` entries there are.
    /// NUMFRAMES
    pub frame_count: u32,

    pub frame_transforms: Option<Vec<FrameTransform>>,

    /// FRAMETRANSFORM
    pub legacy_frame_transforms: Option<Vec<LegacyFrameTransform>>,
}

impl FragmentParser for TrackDef {
    type T = Self;

    const TYPE_ID: u32 = 0x12;
    const TYPE_NAME: &'static str = "TrackDef";

    fn parse(input: &[u8]) -> WResult<'_, TrackDef> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = le_u32(i)?;
        let (i, frame_count) = le_u32(i)?;
        let (i, frame_transforms, legacy_frame_transforms) = if flags & 0x08 == 0x08 {
            let (i, frame_transforms) =
                count(FrameTransform::parse, frame_count as usize).parse(i)?;
            (i, Some(frame_transforms), None)
        } else {
            let (i, legacy_frame_transforms) =
                count(LegacyFrameTransform::parse, frame_count as usize).parse(i)?;
            (i, None, Some(legacy_frame_transforms))
        };

        Ok((
            i,
            TrackDef {
                name_reference,
                flags,
                frame_count,
                frame_transforms,
                legacy_frame_transforms,
            },
        ))
    }
}

impl Fragment for TrackDef {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.frame_count.to_le_bytes()[..],
            &self.frame_transforms.as_ref().map_or(vec![], |ft| {
                ft.iter().flat_map(|f| f.into_bytes()).collect()
            })[..],
            &self.legacy_frame_transforms.as_ref().map_or(vec![], |ft| {
                ft.iter().flat_map(|f| f.into_bytes()).collect()
            })[..],
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
pub struct FrameTransform {
    /// This represents the denominator for the piece’s X, Y, and Z rotation values.
    /// It’s vital to note that it is possible to encounter situations where this value is zero.
    /// I have seen this for pieces with no vertices or polygons and in this case rotation should
    /// be ignored (just use the existing rotation value as passed from the parent piece). My belief
    /// is that such pieces represent attachment points for weapons or items (e.g. shields) and
    /// otherwise don’t represent a part of the model to be rendered.
    pub rotate_denominator: i16,

    /// The numerator for rotation about the X axis.
    pub rotate_x_numerator: i16,

    /// The numerator for rotation about the Y axis.
    pub rotate_y_numerator: i16,

    /// The numerator for rotation about the Z axis.
    pub rotate_z_numerator: i16,

    /// The numerator for translation along the X axis.
    pub shift_x_numerator: i16,

    /// The numerator for translation along the Y axis.
    pub shift_y_numerator: i16,

    /// The numerator for translation along the Z axis.
    pub shift_z_numerator: i16,

    /// The denominator for the piece X, Y, and Z shift values. Like the rotation denominator,
    /// software should check to see if this is zero and ignore translation in that case.
    pub shift_denominator: i16,
}

impl FrameTransform {
    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, rotate_denominator) = le_i16(input)?;
        let (i, rotate_x_numerator) = le_i16(i)?;
        let (i, rotate_y_numerator) = le_i16(i)?;
        let (i, rotate_z_numerator) = le_i16(i)?;
        let (i, shift_x_numerator) = le_i16(i)?;
        let (i, shift_y_numerator) = le_i16(i)?;
        let (i, shift_z_numerator) = le_i16(i)?;
        let (i, shift_denominator) = le_i16(i)?;

        Ok((
            i,
            Self {
                rotate_denominator,
                rotate_x_numerator,
                rotate_y_numerator,
                rotate_z_numerator,
                shift_x_numerator,
                shift_y_numerator,
                shift_z_numerator,
                shift_denominator,
            },
        ))
    }
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.rotate_denominator.to_le_bytes()[..],
            &self.rotate_x_numerator.to_le_bytes()[..],
            &self.rotate_y_numerator.to_le_bytes()[..],
            &self.rotate_z_numerator.to_le_bytes()[..],
            &self.shift_x_numerator.to_le_bytes()[..],
            &self.shift_y_numerator.to_le_bytes()[..],
            &self.shift_z_numerator.to_le_bytes()[..],
            &self.shift_denominator.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// When compressed from ascii the rotation is converted to a quaternion
/// The ascii representation is euler angles out of 512
pub struct LegacyFrameTransform {
    /// The x component of the rotation quaternion.
    pub rotate_x: f32,

    /// The y component of the rotation quaternion.
    pub rotate_y: f32,

    /// The z component of the rotation quaternion.
    pub rotate_z: f32,

    /// The w component of the rotation quaternion.
    pub rotate_w: f32,

    /// The numerator for translation along the X axis.
    pub shift_x_numerator: f32,

    /// The numerator for translation along the Y axis.
    pub shift_y_numerator: f32,

    /// The numerator for translation along the Z axis.
    pub shift_z_numerator: f32,

    /// The denominator for the piece X, Y, and Z shift values.
    /// Software should check to see if this is zero and ignore translation in that case.
    pub shift_denominator: f32,
}

impl LegacyFrameTransform {
    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, shift_denominator) = le_f32(input)?;
        let (i, shift_x_numerator) = le_f32(i)?;
        let (i, shift_y_numerator) = le_f32(i)?;
        let (i, shift_z_numerator) = le_f32(i)?;
        let (i, rotate_w) = le_f32(i)?;
        let (i, rotate_x) = le_f32(i)?;
        let (i, rotate_y) = le_f32(i)?;
        let (i, rotate_z) = le_f32(i)?;

        Ok((
            i,
            Self {
                shift_denominator,
                shift_x_numerator,
                shift_y_numerator,
                shift_z_numerator,
                rotate_w,
                rotate_x,
                rotate_y,
                rotate_z,
            },
        ))
    }
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.shift_denominator.to_le_bytes()[..],
            &self.shift_x_numerator.to_le_bytes()[..],
            &self.shift_y_numerator.to_le_bytes()[..],
            &self.shift_z_numerator.to_le_bytes()[..],
            &self.rotate_w.to_le_bytes()[..],
            &self.rotate_x.to_le_bytes()[..],
            &self.rotate_y.to_le_bytes()[..],
            &self.rotate_z.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0006-0x12.frag")[..];
        let frag = TrackDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-61));
        assert_eq!(frag.flags, 0x8);
        assert_eq!(frag.frame_count, 1);
        assert_eq!(
            frag.frame_transforms,
            Some(vec![FrameTransform {
                rotate_denominator: 16384,
                rotate_x_numerator: 0,
                rotate_y_numerator: 0,
                rotate_z_numerator: 0,
                shift_x_numerator: 0,
                shift_y_numerator: 0,
                shift_z_numerator: 0,
                shift_denominator: 256,
            }])
        );
        assert_eq!(frag.legacy_frame_transforms, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0006-0x12.frag")[..];
        let frag = TrackDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }

    #[test]
    fn it_parses_eq_beta() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip_beta/0652-0x12.frag")[..];
        let frag = TrackDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-7183));
        assert_eq!(frag.flags, 0x0);
        assert_eq!(frag.frame_count, 12);
        assert_eq!(frag.frame_transforms, None);
        assert_eq!(
            frag.legacy_frame_transforms.unwrap()[11],
            LegacyFrameTransform {
                rotate_x: 0.49999997,
                rotate_y: 0.49999997,
                rotate_z: 0.49999997,
                rotate_w: 0.49999997,
                shift_x_numerator: 0.8134234,
                shift_y_numerator: 0.10555774,
                shift_z_numerator: -0.18399855,
                shift_denominator: 1.0
            }
        );
    }

    #[test]
    fn it_serializes_eq_beta() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip_beta/0652-0x12.frag")[..];
        let frag = TrackDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }

    #[test]
    fn it_parses_rtk() {
        let data = &include_bytes!("../../../fixtures/fragments/rtk/0002-0x12.frag")[..];
        let frag = TrackDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-46));
        assert_eq!(frag.flags, 0x0);
        assert_eq!(frag.frame_count, 369);
        assert_eq!(frag.frame_transforms, None);
        assert_eq!(
            frag.legacy_frame_transforms.unwrap()[368],
            LegacyFrameTransform {
                rotate_x: -0.01825354,
                rotate_y: 0.012494401,
                rotate_z: -0.012042822,
                rotate_w: 0.99968284,
                shift_x_numerator: 0.0,
                shift_y_numerator: 0.0,
                shift_z_numerator: 44.48,
                shift_denominator: 0.999999
            }
        );
    }

    #[test]
    fn it_serializes_rtk() {
        let data = &include_bytes!("../../../fixtures/fragments/rtk/0002-0x12.frag")[..];
        let frag = TrackDef::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
