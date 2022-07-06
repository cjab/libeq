use std::any::Any;

use super::{Fragment, FragmentParser, StringReference};

use nom::number::complete::{le_i16, le_u32};
use nom::sequence::tuple;
use nom::IResult;

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
pub struct MobSkeletonPieceTrackFragment {
    pub name_reference: StringReference,

    /// Most flags are _unknown_.
    /// * bit 3 - If set then `data2` exists (though I’m not at all sure about this since I
    ///           have yet to see an example). It could instead mean that the rotation and
    ///           shift entries are `u32`s or it could mean that they’re `f32`s.
    pub flags: u32,

    /// The number of `data1` and `data2` entries there are.
    pub size: u32,

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

    /// _Unknown_ - There are (4 x Size) DWORDs here. This field exists only if the proper bit
    /// in Flags is set. It’s possible that this is a bogus field and really just represents
    /// the above fields in some sort of 32-bit form
    pub data2: Option<Vec<u8>>,
}

impl FragmentParser for MobSkeletonPieceTrackFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x12;
    const TYPE_NAME: &'static str = "MobSkeletonPieceTrack";

    fn parse(input: &[u8]) -> IResult<&[u8], MobSkeletonPieceTrackFragment> {
        let (
            i,
            (
                name_reference,
                flags,
                size,
                rotate_denominator,
                rotate_x_numerator,
                rotate_y_numerator,
                rotate_z_numerator,
                shift_x_numerator,
                shift_y_numerator,
                shift_z_numerator,
                shift_denominator,
            ),
        ) = tuple((
            StringReference::parse,
            le_u32,
            le_u32,
            le_i16,
            le_i16,
            le_i16,
            le_i16,
            le_i16,
            le_i16,
            le_i16,
            le_i16,
        ))(input)?;

        let (remaining, data2) = if i.len() > 0 && (flags & 0x08 == 0x08) {
            (&i[0..0], Some(i.to_vec()))
        } else {
            (i, None)
        };
        if remaining.len() > 0 {
            panic!(
                "Data2 of MobSkeletonPieceTrackFragment found - flags: {:?}, size: {:?}, len: {:?}, remaining: {:?}",
                flags, size, remaining.len(), remaining
            );
        }

        //let (remaining, data2) = if flags & 0x08 == 0x08 {
        //    count(le_i32, (size * 4) as usize)(i).map(|(i, data2)| (i, Some(data2)))?
        //} else {
        //    (i, None)
        //};

        Ok((
            remaining,
            MobSkeletonPieceTrackFragment {
                name_reference,
                flags,
                size,
                rotate_denominator,
                rotate_x_numerator,
                rotate_y_numerator,
                rotate_z_numerator,
                shift_x_numerator,
                shift_y_numerator,
                shift_z_numerator,
                shift_denominator,
                data2,
            },
        ))
    }
}

impl Fragment for MobSkeletonPieceTrackFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.size.to_le_bytes()[..],
            &self.rotate_denominator.to_le_bytes()[..],
            &self.rotate_x_numerator.to_le_bytes()[..],
            &self.rotate_y_numerator.to_le_bytes()[..],
            &self.rotate_z_numerator.to_le_bytes()[..],
            &self.shift_x_numerator.to_le_bytes()[..],
            &self.shift_y_numerator.to_le_bytes()[..],
            &self.shift_z_numerator.to_le_bytes()[..],
            &self.shift_denominator.to_le_bytes()[..],
            &self.data2.as_ref().map_or(vec![], |d| d.to_vec())[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0006-0x12.frag")[..];
        let frag = MobSkeletonPieceTrackFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-61));
        assert_eq!(frag.flags, 0x8);
        assert_eq!(frag.size, 1);
        assert_eq!(frag.rotate_denominator, 16384);
        assert_eq!(frag.rotate_x_numerator, 0);
        assert_eq!(frag.rotate_y_numerator, 0);
        assert_eq!(frag.rotate_z_numerator, 0);
        assert_eq!(frag.shift_x_numerator, 0);
        assert_eq!(frag.shift_y_numerator, 0);
        assert_eq!(frag.shift_z_numerator, 0);
        assert_eq!(frag.shift_denominator, 256);
        assert_eq!(frag.data2, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0006-0x12.frag")[..];
        let frag = MobSkeletonPieceTrackFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
