use std::any::Any;
use std::fmt;

use super::{Fragment, FragmentParser, FragmentRef, StringReference, TextureReferenceFragment};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Copy, Clone)]
pub struct TransparencyFlags(u32);

impl TransparencyFlags {
    const VISIBLE: u32 = 0x80000001;
    const MASKED: u32 = 0x2;
    const OPACITY: u32 = 0x4;
    const TRANSPARENCY: u32 = 0x8;
    const MASKED_OPAQUE: u32 = 0x10;
    /// Found on materials using 'stumpbark' texture - unknown meaning
    /// const UNKNOWN_STUMP_BARK: u32 = 0x540;

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    /// If this material is visible.  If not, this mesh is probably used for collisions only.
    pub fn is_visible(&self) -> bool {
        self.0 & Self::VISIBLE == Self::VISIBLE
    }

    /// This material is masked by a key color
    pub fn has_mask_or_transparency(&self) -> bool {
        self.0 & Self::MASKED == Self::MASKED
    }

    /// This material is alpha blended with a uniform value (e.g. water)
    pub fn has_opacity(&self) -> bool {
        self.0 & Self::OPACITY == Self::OPACITY
    }

    /// This material is opaque but masked by a key color (e.g. tree leaves)
    pub fn has_mask_opaque(&self) -> bool {
        self.0 & Self::MASKED_OPAQUE == Self::MASKED_OPAQUE
    }

    /// This material is alpha blended with red channel (e.g. fire)
    pub fn has_transparency(&self) -> bool {
        self.0 & Self::TRANSPARENCY == Self::TRANSPARENCY
    }

    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Debug for TransparencyFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TransparencyFlags [{:b}]", self.0)
    }
}

impl From<TransparencyFlags> for u32 {
    fn from(value: TransparencyFlags) -> Self {
        value.0
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
///
/// **Type ID:** 0x30
pub struct MaterialFragment {
    pub name_reference: StringReference,

    /// Most flags are _unknown_, however:
    /// * bit 1 - If set then the `pair` field exists. This is usually set.
    pub flags: u32,

    /// Most flags are _unknown_, however:
    // Bit 0 ........ Apparently must be 1 if the texture isn’t transparent.
    // Bit 1 ........ Set to 1 if the texture is masked (e.g. tree leaves).
    // Bit 2 ........ Set to 1 if the texture is semi-transparent but not masked.
    // Bit 3 ........ Set to 1 if the texture is masked and semi-transparent.
    // Bit 4 ........ Set to 1 if the texture is masked but not semi-transparent.
    // Bit 31 ...... Apparently must be 1 if the texture isn’t transparent.
    pub transparency_flags: TransparencyFlags,

    /// This typically contains 0x004E4E4E but has also bee known to contain 0xB2B2B2.
    /// Could this be an RGB reflectivity value?
    pub params2: u32,

    /// Coordinate in the texture with the mask color
    pub mask_color_coord: (f32, f32),

    /// A reference to a [TextureReferenceFragment] fragment.
    pub reference: FragmentRef<TextureReferenceFragment>,

    /// _Unknown_ - This only exists if bit 1 of flags is set. Both fields usually contain 0.
    pub pair: Option<(u32, f32)>,
}

impl FragmentParser for MaterialFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x30;
    const TYPE_NAME: &'static str = "Material";

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialFragment> {
        let (i, (name_reference, flags, transparency_flags, params2, mask_color_coord, reference)) =
            tuple((
                StringReference::parse,
                le_u32,
                TransparencyFlags::parse,
                le_u32,
                tuple((le_f32, le_f32)),
                FragmentRef::parse,
            ))(input)?;

        let (remaining, pair) = if flags & 0x2 == 0x2 {
            tuple((le_u32, le_f32))(i).map(|(rem, p)| (rem, Some(p)))?
        } else {
            (i, None)
        };

        Ok((
            remaining,
            MaterialFragment {
                name_reference,
                flags,
                transparency_flags,
                params2,
                mask_color_coord,
                reference,
                pair,
            },
        ))
    }
}

impl Fragment for MaterialFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.transparency_flags.into_bytes()[..],
            &self.params2.to_le_bytes()[..],
            &self.mask_color_coord.0.to_le_bytes()[..],
            &self.mask_color_coord.1.to_le_bytes()[..],
            &self.reference.into_bytes()[..],
            &self
                .pair
                .map_or(vec![], |p| [p.0.to_le_bytes(), p.1.to_le_bytes()].concat())[..],
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
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0004-0x30.frag")[..];
        let frag = MaterialFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-22));
        assert_eq!(frag.flags, 0x02);
        assert_eq!(frag.transparency_flags.0, 0x80000001);
        assert_eq!(frag.params2, 0x4e4e4e);
        assert_eq!(frag.mask_color_coord, (0.0, 0.75));
        assert_eq!(frag.reference, FragmentRef::new(4));
        assert_eq!(frag.pair, Some((0, 0.0)));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0004-0x30.frag")[..];
        let frag = MaterialFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
