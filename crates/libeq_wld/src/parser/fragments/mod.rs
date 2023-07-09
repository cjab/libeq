mod dm_sprite_def;
mod ambient_light;
mod blit_sprite_def;
mod blit_sprite;
mod bsp_region;
mod bsp_tree;
mod camera;
mod camera_reference;
mod common;
mod first;
mod four_d_sprite;
mod four_d_sprite_def;
mod light_info;
mod light_source;
mod light_source_reference;
mod material;
mod material_list;
mod mesh;
mod mesh_animated_vertices;
mod mesh_animated_vertices_reference;
mod mesh_reference;
mod mob_skeleton_piece_track;
mod mob_skeleton_piece_track_reference;
mod model;
mod object_location;
mod palette_file;
mod particle_cloud_def;
mod particle_sprite;
mod particle_sprite_def;
mod polygon_animation;
mod polygon_animation_reference;
mod region_flag;
mod skeleton_track_set;
mod skeleton_track_set_reference;
mod sphere_list;
mod sphere_list_def;
mod simple_sprite_def;
mod bm_info;
mod texture_images_rtk;
mod simple_sprite;
mod sprite_2d_def;
mod sprite_2d;
mod unknown_0x2e;
mod vertex_color;
mod vertex_color_reference;
mod world_vertices;
mod zone_unknown;

use std::any::Any;
use std::marker::PhantomData;
use std::ops::Deref;

use nom::number::complete::le_i32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{StringReference, WResult};

pub use dm_sprite_def::*;
pub use ambient_light::*;
pub use blit_sprite_def::*;
pub use blit_sprite::*;
pub use bsp_region::*;
pub use bsp_tree::*;
pub use camera::*;
pub use camera_reference::*;
pub use common::*;
pub use first::*;
pub use four_d_sprite::*;
pub use four_d_sprite_def::*;
pub use light_info::*;
pub use light_source::*;
pub use light_source_reference::*;
pub use material::*;
pub use material_list::*;
pub use mesh::*;
pub use mesh_animated_vertices::*;
pub use mesh_animated_vertices_reference::*;
pub use mesh_reference::*;
pub use mob_skeleton_piece_track::*;
pub use mob_skeleton_piece_track_reference::*;
pub use model::*;
pub use object_location::*;
pub use palette_file::*;
pub use particle_cloud_def::*;
pub use particle_sprite::*;
pub use particle_sprite_def::*;
pub use polygon_animation::*;
pub use polygon_animation_reference::*;
pub use region_flag::*;
pub use skeleton_track_set::*;
pub use skeleton_track_set_reference::*;
pub use sphere_list::*;
pub use sphere_list_def::*;
pub use simple_sprite_def::*;
pub use bm_info::*;
pub use texture_images_rtk::*;
pub use simple_sprite::*;
pub use sprite_2d_def::*;
pub use sprite_2d::*;
pub use unknown_0x2e::*;
pub use vertex_color::*;
pub use vertex_color_reference::*;
pub use world_vertices::*;
pub use zone_unknown::*;

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

    pub fn parse(input: &[u8]) -> WResult<FragmentRef<T>> {
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
    fn parse(input: &[u8]) -> WResult<Self::T>;
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub enum FragmentType {
    DmSpriteDef(DmSpriteDef),
    AmbientLight(AmbientLightFragment),
    BlitSpriteDef(BlitSpriteDef),
    BlitSprite(BlitSprite),
    BspRegion(BspRegionFragment),
    BspTree(BspTreeFragment),
    Camera(CameraFragment),
    CameraReference(CameraReferenceFragment),
    First(FirstFragment),
    FourDSprite(FourDSpriteFragment),
    FourDSpriteDef(FourDSpriteDefFragment),
    LightInfo(LightInfoFragment),
    LightSource(LightSourceFragment),
    LightSourceReference(LightSourceReferenceFragment),
    Material(MaterialFragment),
    MaterialList(MaterialListFragment),
    Mesh(MeshFragment),
    MeshAnimatedVertices(MeshAnimatedVerticesFragment),
    MeshAnimatedVerticesReference(MeshAnimatedVerticesReferenceFragment),
    MeshReference(MeshReferenceFragment),
    MobSkeletonPieceTrack(MobSkeletonPieceTrackFragment),
    MobSkeletonPieceTrackReference(MobSkeletonPieceTrackReferenceFragment),
    Model(ModelFragment),
    ObjectLocation(ObjectLocationFragment),
    ParticleSprite(ParticleSpriteFragment),
    ParticleSpriteDef(ParticleSpriteDefFragment),
    ParticleCloudDef(ParticleCloudDefFragment),
    PaletteFile(PaletteFileFragment),
    PolygonAnimation(PolygonAnimationFragment),
    PolygonAnimationReference(PolygonAnimationReferenceFragment),
    RegionFlag(RegionFlagFragment),
    SkeletonTrackSet(SkeletonTrackSetFragment),
    SkeletonTrackSetReference(SkeletonTrackSetReferenceFragment),
    SphereList(SphereListFragment),
    SphereListDef(SphereListDefFragment),
    SimpleSpriteDef(SimpleSpriteDef),
    BmInfo(BmInfo),
    TextureImagesRtk(TextureImagesRtkFragment),
    SimpleSprite(SimpleSprite),
    Sprite2DDef(Sprite2DDef),
    Sprite2D(Sprite2D),
    Unknown0x2e(Unknown0x2eFragment),
    VertexColor(VertexColorFragment),
    VertexColorReference(VertexColorReferenceFragment),
    WorldVertices(WorldVerticesFragment),
    ZoneUnknown(ZoneUnknownFragment),
}

impl Deref for FragmentType {
    type Target = dyn Fragment;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::DmSpriteDef(x) => x,
            Self::AmbientLight(x) => x,
            Self::BlitSpriteDef(x) => x,
            Self::BlitSprite(x) => x,
            Self::BspRegion(x) => x,
            Self::BspTree(x) => x,
            Self::Camera(x) => x,
            Self::CameraReference(x) => x,
            Self::First(x) => x,
            Self::FourDSprite(x) => x,
            Self::FourDSpriteDef(x) => x,
            Self::LightInfo(x) => x,
            Self::LightSource(x) => x,
            Self::LightSourceReference(x) => x,
            Self::Material(x) => x,
            Self::MaterialList(x) => x,
            Self::Mesh(x) => x,
            Self::MeshAnimatedVertices(x) => x,
            Self::MeshAnimatedVerticesReference(x) => x,
            Self::MeshReference(x) => x,
            Self::MobSkeletonPieceTrack(x) => x,
            Self::MobSkeletonPieceTrackReference(x) => x,
            Self::Model(x) => x,
            Self::ObjectLocation(x) => x,
            Self::ParticleSprite(x) => x,
            Self::ParticleSpriteDef(x) => x,
            Self::ParticleCloudDef(x) => x,
            Self::PaletteFile(x) => x,
            Self::PolygonAnimation(x) => x,
            Self::PolygonAnimationReference(x) => x,
            Self::RegionFlag(x) => x,
            Self::SkeletonTrackSet(x) => x,
            Self::SkeletonTrackSetReference(x) => x,
            Self::SphereList(x) => x,
            Self::SphereListDef(x) => x,
            Self::SimpleSpriteDef(x) => x,
            Self::BmInfo(x) => x,
            Self::TextureImagesRtk(x) => x,
            Self::SimpleSprite(x) => x,
            Self::Sprite2DDef(x) => x,
            Self::Sprite2D(x) => x,
            Self::Unknown0x2e(x) => x,
            Self::VertexColor(x) => x,
            Self::VertexColorReference(x) => x,
            Self::WorldVertices(x) => x,
            Self::ZoneUnknown(x) => x,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FragmentGame {
    EverQuest,
    Tanarus,
    ReturnToKrondor,
}
