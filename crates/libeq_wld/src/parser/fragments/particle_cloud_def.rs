use std::any::Any;

use nom::{
    Parser,
    number::complete::{le_f32, le_u8, le_u32},
};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

use super::{BlitSpriteDef, Fragment, FragmentParser, FragmentRef, StringReference, WResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq)]
/// ParticleCloudDef
///
/// Possible unknowns: gravity, location vec3, bbox
///
/// **Type ID:** 0x34
pub struct ParticleCloudDef {
    pub name_reference: StringReference,

    /// zaela: setting 0
    pub unknown_1: u32,

    /// zaela: setting 1
    pub unknown_2: u32,

    /// zaela: Mode
    /// 1: Sphere Dispersal, 2: Plane Dispersal, 3: Stream Dispersal, 4: No Movement
    pub particle_movement: ParticleMovement,

    /// zaela: Flags
    /// Flag 1, High Opacity, Flag 3, Follows Item
    pub flags: u32,

    /// zaela: Simultaneous Particles
    pub simultaneous_particles: u32,

    /// float unknownA_0
    pub unknown_6: u32,

    /// float unknownA_1
    pub unknown_7: u32,

    /// float unknownA_2
    pub unknown_8: u32,

    /// float unknownA_3
    pub unknown_9: u32,

    /// float unknownA_4
    pub unknown_10: u32,

    /// zaela: Sphere Radius
    pub spawn_radius: f32,

    /// zaela: Cone Angle
    pub spawn_angle: f32,

    /// zaela: Lifetime
    pub spawn_lifespan: u32,

    /// zaela: Dispersal Velocity
    pub spawn_velocity: f32,

    /// zaela: Vector Z
    pub spawn_normal_z: f32,

    /// zaela: Vector X
    pub spawn_normal_x: f32,

    /// zaela: Vector Y
    pub spawn_normal_y: f32,

    /// zaela: Emission Delay
    pub spawn_rate: u32,

    pub spawn_scale: f32,

    /// Color, BGRX
    pub color: (u8, u8, u8, u8),

    pub blitsprite: FragmentRef<BlitSpriteDef>,
}

impl FragmentParser for ParticleCloudDef {
    type T = Self;

    const TYPE_ID: u32 = 0x34;
    const TYPE_NAME: &'static str = "ParticleCloudDef";

    fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (i, name_reference) = StringReference::parse(input)?;
        let (i, unknown_1) = le_u32(i)?;
        let (i, unknown_2) = le_u32(i)?;
        let (i, particle_movement) = ParticleMovement::parse(i)?;
        let (i, flags) = le_u32(i)?;
        let (i, simultaneous_particles) = le_u32(i)?;
        let (i, unknown_6) = le_u32(i)?;
        let (i, unknown_7) = le_u32(i)?;
        let (i, unknown_8) = le_u32(i)?;
        let (i, unknown_9) = le_u32(i)?;
        let (i, unknown_10) = le_u32(i)?;
        let (i, spawn_radius) = le_f32(i)?;
        let (i, spawn_angle) = le_f32(i)?;
        let (i, spawn_lifespan) = le_u32(i)?;
        let (i, spawn_velocity) = le_f32(i)?;
        let (i, spawn_normal_z) = le_f32(i)?;
        let (i, spawn_normal_x) = le_f32(i)?;
        let (i, spawn_normal_y) = le_f32(i)?;
        let (i, spawn_rate) = le_u32(i)?;
        let (i, spawn_scale) = le_f32(i)?;
        let (i, color) = (le_u8, le_u8, le_u8, le_u8).parse(i)?;
        let (i, blitsprite) = FragmentRef::<BlitSpriteDef>::parse(i)?;

        Ok((
            i,
            Self {
                name_reference,
                unknown_1,
                unknown_2,
                particle_movement,
                flags,
                simultaneous_particles,
                unknown_6,
                unknown_7,
                unknown_8,
                unknown_9,
                unknown_10,
                spawn_radius,
                spawn_angle,
                spawn_lifespan,
                spawn_velocity,
                spawn_normal_z,
                spawn_normal_x,
                spawn_normal_y,
                spawn_rate,
                spawn_scale,
                color,
                blitsprite,
            },
        ))
    }
}

impl Fragment for ParticleCloudDef {
    fn to_bytes(&self) -> Vec<u8> {
        [
            &self.name_reference.to_bytes()[..],
            &self.unknown_1.to_le_bytes()[..],
            &self.unknown_2.to_le_bytes()[..],
            &self.particle_movement.to_bytes()[..],
            &self.flags.to_le_bytes()[..],
            &self.simultaneous_particles.to_le_bytes()[..],
            &self.unknown_6.to_le_bytes()[..],
            &self.unknown_7.to_le_bytes()[..],
            &self.unknown_8.to_le_bytes()[..],
            &self.unknown_9.to_le_bytes()[..],
            &self.unknown_10.to_le_bytes()[..],
            &self.spawn_radius.to_le_bytes()[..],
            &self.spawn_angle.to_le_bytes()[..],
            &self.spawn_lifespan.to_le_bytes()[..],
            &self.spawn_velocity.to_le_bytes()[..],
            &self.spawn_normal_z.to_le_bytes()[..],
            &self.spawn_normal_x.to_le_bytes()[..],
            &self.spawn_normal_y.to_le_bytes()[..],
            &self.spawn_rate.to_le_bytes()[..],
            &self.spawn_scale.to_le_bytes()[..],
            &self.color.0.to_le_bytes()[..],
            &self.color.1.to_le_bytes()[..],
            &self.color.2.to_le_bytes()[..],
            &self.color.3.to_le_bytes()[..],
            &self.blitsprite.to_bytes()[..],
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
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq)]
pub enum ParticleMovement {
    Sphere = 0x1,
    Plane = 0x2,
    Stream = 0x3,
    None = 0x4,
}

impl ParticleMovement {
    pub fn parse(input: &[u8]) -> WResult<'_, Self> {
        let (remaining, raw) = le_u32(input)?;

        Ok((remaining, FromPrimitive::from_u32(raw).unwrap()))
    }

    fn to_bytes(self) -> Vec<u8> {
        (self as u32).to_le_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0051-0x34.frag")[..];
        let frag = ParticleCloudDef::parse(data).unwrap().1;

        assert_eq!(frag.name_reference, StringReference::new(-566));
        assert_eq!(frag.unknown_1, 4);
        assert_eq!(frag.unknown_2, 3);
        assert_eq!(frag.particle_movement, ParticleMovement::Stream);
        assert_eq!(frag.flags, 0x30500);
        assert_eq!(frag.simultaneous_particles, 30);
        assert_eq!(frag.unknown_6, 0);
        assert_eq!(frag.unknown_7, 0);
        assert_eq!(frag.unknown_8, 0);
        assert_eq!(frag.unknown_9, 0);
        assert_eq!(frag.unknown_10, 0);
        assert_eq!(frag.spawn_radius, 0.1);
        assert_eq!(frag.spawn_angle, 64.0);
        assert_eq!(frag.spawn_lifespan, 2500);
        assert_eq!(frag.spawn_velocity, 1.0);
        assert_eq!(frag.spawn_normal_z, 0.0);
        assert_eq!(frag.spawn_normal_x, -1.0);
        assert_eq!(frag.spawn_normal_y, 0.0);
        assert_eq!(frag.spawn_rate, 83);
        assert_eq!(frag.spawn_scale, 0.5);
        assert_eq!(frag.color, (0x80, 0x80, 0x80, 0x3f));
        assert_eq!(frag.blitsprite, FragmentRef::new(51));
    }

    #[test]
    fn it_serializes() {
        let data = &include_bytes!("../../../fixtures/fragments/gequip/0051-0x34.frag")[..];
        let frag = ParticleCloudDef::parse(data).unwrap().1;

        assert_eq!(&frag.to_bytes()[..], data);
    }
}
