mod actor;
mod actor_def;
mod ambient_light;
mod blit_sprite;
mod blit_sprite_def;
mod bm_info;
mod bm_info_rtk;
mod common;
mod default_palette_file;
mod directional_light;
mod dm_rgb_track;
mod dm_rgb_track_def;
mod dm_sprite;
mod dm_sprite_def;
mod dm_sprite_def_2;
mod dm_track;
mod dm_track_def;
mod dm_track_def_2;
mod global_ambient_light_def;
mod hierarchical_sprite;
mod hierarchical_sprite_def;
mod light;
mod light_def;
mod material_def;
mod material_palette;
mod particle_cloud_def;
mod particle_sprite;
mod particle_sprite_def;
mod point_light;
mod polyhedron;
mod polyhedron_def;
mod region;
mod simple_sprite;
mod simple_sprite_def;
mod sphere;
mod sphere_list;
mod sphere_list_def;
mod sprite_2d;
mod sprite_2d_def;
mod sprite_3d;
mod sprite_3d_def;
mod sprite_4d;
mod sprite_4d_def;
mod track;
mod track_def;
mod world_tree;
mod world_vertices;
mod zone;

use std::any::Any;
use std::marker::PhantomData;
use std::ops::Deref;

use nom::number::complete::le_i32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{StringReference, WResult};

pub use actor::*;
pub use actor_def::*;
pub use ambient_light::*;
pub use blit_sprite::*;
pub use blit_sprite_def::*;
pub use bm_info::*;
pub use bm_info_rtk::*;
pub use common::*;
pub use default_palette_file::*;
pub use directional_light::*;
pub use dm_rgb_track::*;
pub use dm_rgb_track_def::*;
pub use dm_sprite::*;
pub use dm_sprite_def::*;
pub use dm_sprite_def_2::*;
pub use dm_track::*;
pub use dm_track_def::*;
pub use dm_track_def_2::*;
pub use global_ambient_light_def::*;
pub use hierarchical_sprite::*;
pub use hierarchical_sprite_def::*;
pub use light::*;
pub use light_def::*;
pub use material_def::*;
pub use material_palette::*;
pub use particle_cloud_def::*;
pub use particle_sprite::*;
pub use particle_sprite_def::*;
pub use point_light::*;
pub use polyhedron::*;
pub use polyhedron_def::*;
pub use region::*;
pub use simple_sprite::*;
pub use simple_sprite_def::*;
pub use sphere::*;
pub use sphere_list::*;
pub use sphere_list_def::*;
pub use sprite_2d::*;
pub use sprite_2d_def::*;
pub use sprite_3d::*;
pub use sprite_3d_def::*;
pub use sprite_4d::*;
pub use sprite_4d_def::*;
pub use track::*;
pub use track_def::*;
pub use world_tree::*;
pub use world_vertices::*;
pub use zone::*;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FragmentRef<T> {
    Name(StringReference, PhantomData<T>),
    Index(u32, PhantomData<T>),
}

impl<T> FragmentRef<T> {
    pub fn new(idx: i32) -> FragmentRef<T> {
        if idx > 0 {
            FragmentRef::Index(idx as u32, PhantomData)
        } else {
            let name_ref = StringReference::new(idx);
            FragmentRef::Name(name_ref, PhantomData)
        }
    }

    pub fn parse(input: &[u8]) -> WResult<'_, FragmentRef<T>> {
        let (remaining, frag_ref_idx) = le_i32(input)?;
        Ok((remaining, FragmentRef::new(frag_ref_idx)))
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        match self {
            Self::Name(string_ref, _) => string_ref.into_bytes(),
            Self::Index(idx, _) => idx.to_le_bytes().to_vec(),
        }
    }
}

pub trait Fragment {
    fn into_bytes(&self) -> Vec<u8>;
    fn as_any(&self) -> &dyn Any;
    fn name_ref(&self) -> &StringReference;
    fn type_id(&self) -> u32;
}

pub trait FragmentParser {
    type T;
    const TYPE_ID: u32;
    const TYPE_NAME: &'static str;
    fn parse(input: &[u8]) -> WResult<'_, Self::T>;
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum FragmentType {
    DmSpriteDef(DmSpriteDef),
    AmbientLight(AmbientLight),
    BlitSpriteDef(BlitSpriteDef),
    BlitSprite(BlitSprite),
    Region(Region),
    WorldTree(WorldTree),
    Sprite3DDef(Sprite3DDef),
    Sprite3D(Sprite3D),
    GlobalAmbientLightDef(GlobalAmbientLightDef),
    Sprite4D(Sprite4D),
    Sprite4DDef(Sprite4DDef),
    PointLight(PointLight),
    LightDef(LightDef),
    Light(Light),
    MaterialDef(MaterialDef),
    MaterialPalette(MaterialPalette),
    DmSpriteDef2(DmSpriteDef2),
    DmTrackDef2(DmTrackDef2),
    DmTrack(DmTrack),
    DmSprite(DmSprite),
    TrackDef(TrackDef),
    Track(Track),
    ActorDef(ActorDef),
    Actor(Actor),
    ParticleSprite(ParticleSprite),
    ParticleSpriteDef(ParticleSpriteDef),
    ParticleCloudDef(ParticleCloudDef),
    DefaultPaletteFile(DefaultPaletteFile),
    PolyhedronDef(PolyhedronDef),
    Polyhedron(Polyhedron),
    Zone(Zone),
    HierarchicalSpriteDef(HierarchicalSpriteDef),
    HierarchicalSprite(HierarchicalSprite),
    SphereList(SphereList),
    SphereListDef(SphereListDef),
    SimpleSpriteDef(SimpleSpriteDef),
    BmInfo(BmInfo),
    BmInfoRtk(BmInfoRtk),
    SimpleSprite(SimpleSprite),
    Sprite2DDef(Sprite2DDef),
    Sprite2D(Sprite2D),
    DmTrackDef(DmTrackDef),
    DmRGBTrackDef(DmRGBTrackDef),
    DmRGBTrack(DmRGBTrack),
    WorldVertices(WorldVertices),
    Sphere(Sphere),
    DirectionalLight(DirectionalLight),
}

impl Deref for FragmentType {
    type Target = dyn Fragment;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::DmSpriteDef(x) => x,
            Self::AmbientLight(x) => x,
            Self::BlitSpriteDef(x) => x,
            Self::BlitSprite(x) => x,
            Self::Region(x) => x,
            Self::WorldTree(x) => x,
            Self::Sprite3DDef(x) => x,
            Self::Sprite3D(x) => x,
            Self::GlobalAmbientLightDef(x) => x,
            Self::Sprite4D(x) => x,
            Self::Sprite4DDef(x) => x,
            Self::PointLight(x) => x,
            Self::LightDef(x) => x,
            Self::Light(x) => x,
            Self::MaterialDef(x) => x,
            Self::MaterialPalette(x) => x,
            Self::DmSpriteDef2(x) => x,
            Self::DmTrackDef2(x) => x,
            Self::DmTrack(x) => x,
            Self::DmSprite(x) => x,
            Self::TrackDef(x) => x,
            Self::Track(x) => x,
            Self::ActorDef(x) => x,
            Self::Actor(x) => x,
            Self::ParticleSprite(x) => x,
            Self::ParticleSpriteDef(x) => x,
            Self::ParticleCloudDef(x) => x,
            Self::DefaultPaletteFile(x) => x,
            Self::PolyhedronDef(x) => x,
            Self::Polyhedron(x) => x,
            Self::Zone(x) => x,
            Self::HierarchicalSpriteDef(x) => x,
            Self::HierarchicalSprite(x) => x,
            Self::SphereList(x) => x,
            Self::SphereListDef(x) => x,
            Self::SimpleSpriteDef(x) => x,
            Self::BmInfo(x) => x,
            Self::BmInfoRtk(x) => x,
            Self::SimpleSprite(x) => x,
            Self::Sprite2DDef(x) => x,
            Self::Sprite2D(x) => x,
            Self::DmTrackDef(x) => x,
            Self::DmRGBTrackDef(x) => x,
            Self::DmRGBTrack(x) => x,
            Self::WorldVertices(x) => x,
            Self::Sphere(x) => x,
            Self::DirectionalLight(x) => x,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FragmentGame {
    EverQuest,
    Tanarus,
    ReturnToKrondor,
}
