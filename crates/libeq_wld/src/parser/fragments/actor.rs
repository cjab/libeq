use std::any::Any;

use crate::parser::strings::{decode_string, encode_string};

use super::common::Location;
use super::{DmRGBTrack, Fragment, FragmentParser, FragmentRef, Sphere, StringReference, WResult};

use nom::Parser;
use nom::multi::count;
use nom::number::complete::{le_f32, le_u8, le_u32};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
/// **Type ID:** 0x15
pub struct Actor {
    pub name_reference: StringReference,

    /// When used in main zone files, the reference points to a 0x14 Player Info fragment. When used for static (placeable) objects,
    /// the reference is a string reference (not a fragment reference) and points to a “magic” string.
    /// It typically contains the name of the object with “_ACTORDEF” appended to the end.
    pub actor_def_reference: StringReference, // FIXME: This can be a FragmentRef sometimes, as stated above

    /// Typically 0x2E when used in main zone files and 0x32E when
    /// used for placeable objects.
    pub flags: ActorInstFlags,

    /// When used in main zone files, points to a 0x16 fragment.
    /// When used for placeable objects, seems to always contain 0.
    /// This might be due to the difference in the Flags value.
    pub sphere_reference: FragmentRef<Sphere>,

    pub current_action: Option<u32>,

    pub location: Option<Location>,

    /// Windcatcher:
    /// When used in main zone files, typically contains 0.5. When used for
    /// placeable objects, contains the object’s scaling factor in the Y direction
    /// (e.g. 2.0 would make the object twice as big in the Y direction).
    /// NEW:
    /// BOUNDINGRADIUS %f
    pub bounding_radius: Option<f32>,

    /// Windcatcher:
    /// When used in main zone files, typically contains 0.5. When used for
    /// placeable objects, contains the object’s scaling factor in the X direction
    /// (e.g. 2.0 would make the object twice as big in the X direction).
    /// NEW:
    /// SCALEFACTOR %f
    pub scale_factor: Option<f32>,

    pub sound_name_reference: Option<StringReference>,

    /// When used in main zone files, typically contains 0 (might be related to
    /// the Flags value). When used for placeable objects, points to a 0x33 Vertex
    /// Color Reference fragment.
    pub vertex_color_reference: Option<FragmentRef<DmRGBTrack>>,

    /// Windcatcher:
    /// Typically contains 30 when used in main zone files and 0 when used for
    /// placeable objects. This field only exists if `vertex_color_reference` points to a fragment.
    /// NEW:
    /// Length of USERDATA string
    pub user_data_size: u32,

    /// USERDATA %s
    pub user_data: String,
}

impl FragmentParser for Actor {
    type T = Self;

    const TYPE_ID: u32 = 0x15;
    const TYPE_NAME: &'static str = "Actor";

    fn parse(input: &[u8]) -> WResult<'_, Actor> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, actor_def_reference) = StringReference::parse(i)?;
        let (i, flags) = ActorInstFlags::parse(i)?;
        let (i, sphere_reference) = FragmentRef::parse(i)?;
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
        let (i, bounding_radius) = if flags.has_bounding_radius() {
            le_f32(i).map(|(i, b)| (i, Some(b)))?
        } else {
            (i, None)
        };
        let (i, scale_factor) = if flags.has_scale_factor() {
            le_f32(i).map(|(i, s)| (i, Some(s)))?
        } else {
            (i, None)
        };
        let (i, sound_name_reference) = if flags.has_sound() {
            StringReference::parse(i).map(|(i, s)| (i, Some(s)))?
        } else {
            (i, None)
        };
        let (i, vertex_color_reference) = if flags.has_vertex_color_reference() {
            FragmentRef::parse(i).map(|(rem, f)| (rem, Some(f)))?
        } else {
            (i, None)
        };
        let (i, user_data_size) = le_u32(i)?;
        let (i, user_data) = count(le_u8, user_data_size as usize).parse(i)?;

        Ok((
            i,
            Actor {
                name_reference,
                actor_def_reference,
                flags,
                sphere_reference,
                current_action,
                location,
                bounding_radius,
                scale_factor,
                sound_name_reference,
                vertex_color_reference,
                user_data_size,
                user_data: decode_string(&user_data).trim_end_matches("\0").to_string(),
            },
        ))
    }
}

impl Fragment for Actor {
    fn into_bytes(&self) -> Vec<u8> {
        let user_data_size = self.user_data_size as usize;
        let padding = (4 - user_data_size % 4) % 4;
        let mut user_data = encode_string(&format!("{}{}", &self.user_data, "\0"));
        user_data.resize(user_data_size + padding, 0);

        [
            &self.name_reference.into_bytes()[..],
            &self.actor_def_reference.into_bytes()[..],
            &self.flags.into_bytes()[..],
            &self.sphere_reference.into_bytes()[..],
            &self
                .current_action
                .map_or(vec![], |a| a.to_le_bytes().to_vec())[..],
            &self.location.as_ref().map_or(vec![], |l| l.into_bytes())[..],
            &self
                .bounding_radius
                .map_or(vec![], |b| b.to_le_bytes().to_vec())[..],
            &self
                .scale_factor
                .map_or(vec![], |s| s.to_le_bytes().to_vec())[..],
            &self.sound_name_reference.map_or(vec![], |s| s.into_bytes())[..],
            &self
                .vertex_color_reference
                .as_ref()
                .map_or(vec![], |m| m.into_bytes())[..],
            &self.user_data_size.to_le_bytes()[..],
            &user_data[..],
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
pub struct ActorInstFlags(u32);

impl ActorInstFlags {
    const HAS_CURRENT_ACTION: u32 = 0x01;
    const HAS_LOCATION: u32 = 0x02;
    const HAS_BOUNDING_RADIUS: u32 = 0x04;
    const HAS_SCALE_FACTOR: u32 = 0x08;
    const HAS_SOUND: u32 = 0x10;
    const ACTIVE: u32 = 0x20;
    const SPRITE_VOLUME_ONLY: u32 = 0x80;
    const HAS_VERTEX_COLOR_REFERENCE: u32 = 0x100;

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (remaining, raw_flags) = le_u32(input)?;
        Ok((remaining, Self(raw_flags)))
    }

    fn into_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }

    pub fn has_current_action(&self) -> bool {
        self.0 & Self::HAS_CURRENT_ACTION == Self::HAS_CURRENT_ACTION
    }

    pub fn has_location(&self) -> bool {
        self.0 & Self::HAS_LOCATION == Self::HAS_LOCATION
    }

    pub fn has_bounding_radius(&self) -> bool {
        self.0 & Self::HAS_BOUNDING_RADIUS == Self::HAS_BOUNDING_RADIUS
    }

    pub fn has_scale_factor(&self) -> bool {
        self.0 & Self::HAS_SCALE_FACTOR == Self::HAS_SCALE_FACTOR
    }

    pub fn has_sound(&self) -> bool {
        self.0 & Self::HAS_SOUND == Self::HAS_SOUND
    }

    /// AppendItemToActiveAIList
    pub fn active(&self) -> bool {
        self.0 & Self::ACTIVE == Self::ACTIVE
    }

    pub fn sprite_volume_only(&self) -> bool {
        self.0 & Self::SPRITE_VOLUME_ONLY == Self::SPRITE_VOLUME_ONLY
    }

    pub fn has_vertex_color_reference(&self) -> bool {
        self.0 & Self::HAS_VERTEX_COLOR_REFERENCE == Self::HAS_VERTEX_COLOR_REFERENCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4641-0x15.frag")[..];
        let (remaining, frag) = Actor::parse(data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(0));
        // FIXME: this is a FragmentRef
        assert_eq!(frag.actor_def_reference, StringReference::new(4640));
        assert_eq!(frag.flags, ActorInstFlags(46));
        assert_eq!(frag.sphere_reference, FragmentRef::new(4641));
        assert_eq!(frag.current_action, None);
        assert_eq!(
            frag.location,
            Some(Location {
                x: -2935.2515,
                y: -2823.1519,
                z: -19.758118,
                rotate_z: 0.0,
                rotate_y: 0.0,
                rotate_x: 0.0,
                unknown: 0
            })
        );
        assert_eq!(frag.bounding_radius, Some(0.5));
        assert_eq!(frag.scale_factor, Some(0.5));
        assert_eq!(frag.sound_name_reference, None);
        assert_eq!(frag.vertex_color_reference, None);
        assert_eq!(frag.user_data_size, 0);
        assert_eq!(frag.user_data, String::new());
        assert_eq!(remaining, vec![]);
    }

    #[test]
    fn it_parses_objects() {
        let data = &include_bytes!("../../../fixtures/fragments/objects/0002-0x15.frag")[..];
        let (remaining, frag) = Actor::parse(data).unwrap();

        assert_eq!(frag.name_reference, StringReference::new(0));
        assert_eq!(frag.actor_def_reference, StringReference::new(-10));
        assert_eq!(frag.flags, ActorInstFlags(814));
        assert_eq!(frag.sphere_reference, FragmentRef::new(0));
        assert_eq!(frag.current_action, None);
        assert_eq!(
            frag.location,
            Some(Location {
                x: -2022.8826,
                y: -2419.7405,
                z: 198.36989,
                rotate_z: 373.0,
                rotate_y: 0.0,
                rotate_x: 0.0,
                unknown: 0
            })
        );
        assert_eq!(frag.bounding_radius, Some(1.0));
        assert_eq!(frag.scale_factor, Some(1.0));
        assert_eq!(frag.sound_name_reference, None);
        assert_eq!(frag.vertex_color_reference, Some(FragmentRef::new(2)));
        assert_eq!(frag.user_data_size, 0);
        assert_eq!(frag.user_data, String::new());
        assert_eq!(remaining, vec![]);
    }

    #[test]
    fn it_parses_userdata() {
        let data =
            &include_bytes!("../../../fixtures/fragments/wldcom/actorinst-userdata-0000-0x15.frag")
                [..];
        let frag = Actor::parse(data).unwrap().1;

        assert_eq!(frag.user_data_size, 5);
        assert_eq!(frag.user_data, String::from("data"));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gfaydark/4641-0x15.frag")[..];
        let frag = Actor::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }

    #[test]
    fn it_serializes_objects() {
        let data = &include_bytes!("../../../fixtures/fragments/objects/0002-0x15.frag")[..];
        let frag = Actor::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }

    #[test]
    fn it_serializes_userdata() {
        let data =
            &include_bytes!("../../../fixtures/fragments/wldcom/actorinst-userdata-0000-0x15.frag")
                [..];
        let frag = Actor::parse(data).unwrap().1;

        assert_eq!(&frag.into_bytes()[..], data);
    }
}
