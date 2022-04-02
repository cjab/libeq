use std::any::Any;

use super::{fragment_ref, Fragment, FragmentRef, FragmentType, TextureReferenceFragment};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
///
/// **Type ID:** 0x30
pub struct MaterialFragment {
    /// Most flags are _unknown_, however:
    /// * bit 1 - If set then the `pair` field exists. This is usually set.
    pub flags: u32,

    /// Most flags are _unknown_, however:
    /// * bit 0 - It seems like this must be set if the texture is not transparent.
    /// * bit 1 - Set if the texture is masked (e.g. tree leaves).
    /// * bit 2 - Set if the texture is semi-transparent but not masked.
    /// * bit 3 - Set if the texture is masked and semi-transparent.
    /// * bit 4 Set if the texture is masked but not semi-transparent.
    /// * bit 31 - It seems like this must be set if the texture is not transparent.
    pub params1: u32,

    /// This typically contains 0x004E4E4E but has also bee known to contain 0xB2B2B2.
    /// Could this be an RGB reflectivity value?
    pub params2: u32,

    /// _Unknown_ - Usually contains 0.
    pub params3: (f32, f32),

    /// A reference to a [TextureReferenceFragment] fragment.
    pub reference: FragmentRef<TextureReferenceFragment>,

    /// _Unknown_ - This only exists if bit 1 of flags is set. Both fields usually contain 0.
    pub pair: Option<(u32, f32)>,
}

impl FragmentType for MaterialFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x30;

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialFragment> {
        let (i, (flags, params1, params2, params3, reference)) = tuple((
            le_u32,
            le_u32,
            le_u32,
            tuple((le_f32, le_f32)),
            fragment_ref,
        ))(input)?;

        let (remaining, pair) = if flags & 0x2 == 0x2 {
            tuple((le_u32, le_f32))(i).map(|(rem, p)| (rem, Some(p)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            MaterialFragment {
                flags,
                params1,
                params2,
                params3,
                reference,
                pair,
            },
        ))
    }
}

impl Fragment for MaterialFragment {
    fn serialize(&self) -> Vec<u8> {
        [
            &self.flags.to_le_bytes()[..],
            &self.params1.to_le_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self.params3.0.to_le_bytes()[..],
            &self.params3.1.to_le_bytes()[..],
            &self.reference.serialize().to_le_bytes()[..],
            &self
                .pair
                .map_or(vec![], |p| [p.0.to_le_bytes(), p.1.to_le_bytes()].concat())[..],
        ]
        .concat()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
