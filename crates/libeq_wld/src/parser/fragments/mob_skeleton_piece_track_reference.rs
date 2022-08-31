use std::any::Any;

use super::{
    Fragment, FragmentParser, FragmentRef, MobSkeletonPieceTrackFragment, StringReference, WResult,
};

use nom::number::complete::le_u32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// A reference to a [MobSkeletonPieceTrackFragment].
///
/// **Type ID:** 0x13
pub struct MobSkeletonPieceTrackReferenceFragment {
    pub name_reference: StringReference,

    /// The [MobSkeletonPieceTrackFragment] reference.
    pub reference: FragmentRef<MobSkeletonPieceTrackFragment>,

    pub flags: TrackInstanceFlags,

    /// SLEEP %d
    pub sleep: Option<u32>,
}

impl FragmentParser for MobSkeletonPieceTrackReferenceFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x13;
    const TYPE_NAME: &'static str = "MobSkeletonPieceTrackReference";

    fn parse(input: &[u8]) -> WResult<MobSkeletonPieceTrackReferenceFragment> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, reference) = FragmentRef::parse(i)?;
        let (i, flags) = TrackInstanceFlags::parse(i)?;
        let (i, sleep) = if flags.has_sleep() {
            le_u32(i).map(|(i, s)| (i, Some(s)))?
        } else {
            (i, None)
        };

        Ok((
            i,
            MobSkeletonPieceTrackReferenceFragment {
                name_reference,
                reference,
                flags,
                sleep,
            },
        ))
    }
}

impl Fragment for MobSkeletonPieceTrackReferenceFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.sleep.map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
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
pub struct TrackInstanceFlags(u32);

impl TrackInstanceFlags {
    const HAS_SLEEP: u32 = 0x01;
    const REVERSE: u32 = 0x02;
    const INTERPOLATE: u32 = 0x04;

    fn parse(input: &[u8]) -> WResult<Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_sleep(&self) -> bool {
        self.0 & Self::HAS_SLEEP == Self::HAS_SLEEP
    }

    pub fn reverse(&self) -> bool {
        self.0 & Self::REVERSE == Self::REVERSE
    }

    pub fn interpolate(&self) -> bool {
        self.0 & Self::INTERPOLATE == Self::INTERPOLATE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0007-0x13.frag")[..];
        let frag = MobSkeletonPieceTrackReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(frag.name_reference, StringReference::new(-75));
        assert_eq!(frag.reference, FragmentRef::new(7));
        assert_eq!(frag.flags.has_sleep(), false);
        assert_eq!(frag.flags.reverse(), false);
        assert_eq!(frag.flags.interpolate(), false);
        assert_eq!(frag.sleep, None);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0007-0x13.frag")[..];
        let frag = MobSkeletonPieceTrackReferenceFragment::parse(data)
            .unwrap()
            .1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
