use std::any::Any;

use super::{Fragment, FragmentParser, StringReference};

use nom::multi::count;
use nom::number::complete::{le_f32, le_u32};
use nom::IResult;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// Static or animated model reference or player info.
///
/// **Type ID:** 0x14
pub struct ModelFragment {
    pub name_reference: StringReference,

    pub flags: ActorDefFlags,

    /// Windcatcher:
    /// This isn’t really a fragment reference but a string reference.
    /// It points to a “magic” string. When this fragment is used in main zone
    /// files the string is “FLYCAMCALLBACK”. When used as a placeable object reference,
    /// the string is “SPRITECALLBACK”. When creating a 0x14 fragment this is currently
    /// accomplished by creating a fragment reference, setting the fragment to null, and
    /// setting the reference name to the magic string.
    pub callback_name_reference: StringReference,

    /// Tells how many action entries there are.
    pub action_count: u32,

    /// Tells how many fragment entries there are.
    pub fragment_reference_count: u32,

    /// SPHERE, SPHERELIST, or POLYHEDRON reference (possibly others!)
    pub bounds_reference: u32,

    pub current_action: Option<u32>,

    pub location: Option<Location>,

    pub actions: Vec<Action>,

    /// There are `fragment_reference_count` fragment references here. These references can point to several different
    /// kinds of fragments. In main zone files, there seems to be only one entry, which points to
    /// a 0x09 Camera Reference fragment. When this is instead a static object reference, the entry
    /// points to either a 0x2D Mesh Reference fragment. If this is an animated (mob) object
    /// reference, it points to a 0x11 Skeleton Track Set Reference fragment.
    /// This also has been seen to point to a 0x07 Two-dimensional Object Reference fragment
    /// (e.g. coins and blood spots).
    pub fragment_references: Vec<u32>,

    pub unknown: u32,
}

impl FragmentParser for ModelFragment {
    type T = Self;

    const TYPE_ID: u32 = 0x14;
    const TYPE_NAME: &'static str = "Model";

    fn parse(input: &[u8]) -> IResult<&[u8], ModelFragment> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, flags) = ActorDefFlags::parse(i)?;
        let (i, callback_name_reference) = StringReference::parse(i)?;
        let (i, action_count) = le_u32(i)?;
        let (i, fragment_reference_count) = le_u32(i)?;
        let (i, bounds_reference) = le_u32(i)?;
        let (i, current_action) = if flags.has_current_action() {
            le_u32(i).map(|(i, c)| (i, Some(c)))?
        } else {
            (i, None)
        };
        let (i, location) = if flags.has_location() {
            Location::parse(i).map(|(i, l)| (i, Some(l)))?
        } else {
            (i, None)
        };
        let (i, actions) = count(Action::parse, action_count as usize)(i)?;
        let (i, fragment_references) = count(le_u32, fragment_reference_count as usize)(i)?;
        let (i, unknown) = le_u32(i)?;

        Ok((
            i,
            Self {
                name_reference,
                flags,
                callback_name_reference,
                action_count,
                fragment_reference_count,
                bounds_reference,
                current_action,
                location,
                actions,
                fragment_references,
                unknown,
            },
        ))
    }
}

impl Fragment for ModelFragment {
    fn into_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.callback_name_reference.into_bytes()[..],
            &self.action_count.to_le_bytes()[..],
            &self.fragment_reference_count.to_le_bytes()[..],
            &self.bounds_reference.to_le_bytes()[..],
            &self
                .current_action
                .map_or(vec![], |c| c.to_le_bytes().to_vec())[..],
            &self.location.as_ref().map_or(vec![], |l| l.into_bytes())[..],
            &self
                .actions
                .iter()
                .flat_map(|a| a.into_bytes())
                .collect::<Vec<_>>()[..],
            &self
                .fragment_references
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<_>>()[..],
            &self.unknown.to_le_bytes()[..],
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
pub struct ActorDefFlags(u32);

impl ActorDefFlags {
    const HAS_CURRENT_ACTION: u32 = 0x01;
    const HAS_LOCATION: u32 = 0x02; //TODO:
    const ACTIVE_GEOMETRY: u32 = 0x40;
    const SPRITE_VOLUME_ONLY: u32 = 0x80;

    fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn sprite_volume_only(&self) -> bool {
        self.0 & Self::SPRITE_VOLUME_ONLY == Self::SPRITE_VOLUME_ONLY
    }

    pub fn active_geometry(&self) -> bool {
        self.0 & Self::ACTIVE_GEOMETRY == Self::ACTIVE_GEOMETRY
    }

    pub fn has_location(&self) -> bool {
        self.0 & Self::HAS_LOCATION == Self::HAS_LOCATION
    }

    pub fn has_current_action(&self) -> bool {
        self.0 & Self::HAS_CURRENT_ACTION == Self::HAS_CURRENT_ACTION
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Represents LOCATION within an ACTORDEF.
pub struct Location {
    /// LOCATION   %d   %f   %f   %f   %d   %d   %d
    /// LOCATION loc6 loc0 loc1 loc2 loc3 loc4 loc5
    pub loc0: f32,
    pub loc1: f32,
    pub loc2: f32,
    pub loc3: f32,
    pub loc4: f32,
    pub loc5: f32,
    pub loc6: u32,
}

impl Location {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, loc0) = le_f32(input)?;
        let (i, loc1) = le_f32(i)?;
        let (i, loc2) = le_f32(i)?;
        let (i, loc3) = le_f32(i)?;
        let (i, loc4) = le_f32(i)?;
        let (i, loc5) = le_f32(i)?;
        let (i, loc6) = le_u32(i)?;

        Ok((
            i,
            Self {
                loc0,
                loc1,
                loc2,
                loc3,
                loc4,
                loc5,
                loc6,
            },
        ))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.loc0.to_le_bytes()[..],
            &self.loc1.to_le_bytes()[..],
            &self.loc2.to_le_bytes()[..],
            &self.loc3.to_le_bytes()[..],
            &self.loc4.to_le_bytes()[..],
            &self.loc5.to_le_bytes()[..],
            &self.loc6.to_le_bytes()[..],
        ]
        .concat()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// Represents ACTION within an ACTORDEF.
pub struct Action {
    pub levels_of_detail_count: u32,
    pub unknown: u32,
    // This is a sequence of minimum distances and maximum distances for each level of detail.
    // MINDISTANCE %f
    // MAXDISTANCE %f
    pub levels_of_detail_distances: Vec<f32>,
}

impl Action {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, levels_of_detail_count) = le_u32(input)?;
        let (i, unknown) = le_u32(i)?;
        let (i, levels_of_detail_distances) = count(le_f32, levels_of_detail_count as usize)(i)?;

        Ok((
            i,
            Self {
                levels_of_detail_count,
                unknown,
                levels_of_detail_distances,
            },
        ))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        [
            &self.levels_of_detail_count.to_le_bytes()[..],
            &self.unknown.to_le_bytes()[..],
            &self
                .levels_of_detail_distances
                .iter()
                .flat_map(|d| d.to_le_bytes())
                .collect::<Vec<_>>()[..],
        ]
        .concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4639-0x14.frag")[..];
        let frag = ModelFragment::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-52594));
        assert_eq!(frag.flags, ActorDefFlags(0));
        assert_eq!(frag.callback_name_reference, StringReference::new(-52579));
        assert_eq!(frag.action_count, 1);
        assert_eq!(frag.fragment_reference_count, 1);
        assert_eq!(frag.bounds_reference, 0);
        assert_eq!(frag.current_action, None);
        assert_eq!(frag.location, None);
        assert_eq!(
            frag.actions,
            vec![Action {
                levels_of_detail_count: 1,
                unknown: 0,
                levels_of_detail_distances: vec![1e30]
            }]
        );
        assert_eq!(frag.fragment_references, vec![4639]);
        assert_eq!(frag.unknown, 0);
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4639-0x14.frag")[..];
        let frag = ModelFragment::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
