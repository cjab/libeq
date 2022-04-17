use std::any::Any;

use super::{Fragment, FragmentParser, FragmentRef, StringReference, TextureReferenceFragment};

use nom::number::complete::{le_f32, le_u32};
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct TransparencyFlags(u32);

impl TransparencyFlags {
    const VISIBLE: u32 = 0x80000001;
    const MASKED: u32 = 0x0000010;
    const TRANSPARENCY: u32 = 0x0000100;
    const MASKED_TRANSPARENCY: u32 = 0x0001000;
    const MASKED_OPAQUE: u32 = 0x0010000;
    /// Found on materials using 'stumpbark' texture - unknown meaning
    /// const UNKNOWN_STUMP_BARK: u32 = 0x540;

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn serialize(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    /// If this material is visible.  If not, this mesh is probably used for collisions only.
    pub fn is_visible(&self) -> bool {
        self.0 & Self::VISIBLE == Self::VISIBLE
    }

    /// If this material is masked by a key color
    pub fn is_masked(&self) -> bool {
        self.0 & Self::MASKED == Self::MASKED
    }

    /// If this material has transparency (e.g. water)
    pub fn is_transparent(&self) -> bool {
        self.0 & Self::TRANSPARENCY == Self::TRANSPARENCY
    }

    /// If this material is opaque but masked by a key color (e.g. tree leaves)
    pub fn is_masked_opaque(&self) -> bool {
        self.0 & Self::MASKED_OPAQUE == Self::MASKED_OPAQUE
    }

    /// If this material is transparent and also masked (e.g. fire)
    pub fn is_masked_transparent(&self) -> bool {
        self.0 & Self::MASKED_TRANSPARENCY == Self::MASKED_TRANSPARENCY
    }
}

#[derive(Debug, PartialEq)]
///
/// **Type ID:** 0x30
pub struct MaterialFragment {
    pub name_reference: StringReference,

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
    pub transparency_flags: TransparencyFlags,

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

impl FragmentParser for MaterialFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x30;
    const TYPE_NAME: &'static str = "Material";

    fn parse(input: &[u8]) -> IResult<&[u8], MaterialFragment> {
        let (i, (name_reference, flags, transparency_flags, params2, params3, reference)) =
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
            &self.name_reference.serialize()[..],
            &self.flags.to_le_bytes()[..],
            &self.transparency_flags.serialize()[..],
            &self.params2.to_le_bytes()[..],
            &self.params3.0.to_le_bytes()[..],
            &self.params3.1.to_le_bytes()[..],
            &self.reference.serialize()[..],
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
        assert_eq!(frag.transparency_flags, 0x80000001);
        assert_eq!(frag.params2, 0x4e4e4e);
        assert_eq!(frag.params3, (0.0, 0.75));
        assert_eq!(frag.reference, FragmentRef::new(4));
        assert_eq!(frag.pair, Some((0, 0.0)));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/0004-0x30.frag")[..];
        let frag = MaterialFragment::parse(data).unwrap().1;

        assert_eq!(&frag.serialize()[..], data);
    }
}
