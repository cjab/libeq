mod alternate_mesh;
mod ambient_light;
mod blit_sprite_definition;
mod blit_sprite_reference;
mod bsp_region;
mod bsp_tree;
mod camera;
mod camera_reference;
mod common;
mod first;
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
mod polygon_animation;
mod polygon_animation_reference;
mod region_flag;
mod skeleton_track_set;
mod skeleton_track_set_reference;
mod sphere_list;
mod sphere_list_def;
mod texture;
mod texture_images;
mod texture_reference;
mod two_dimensional_object;
mod two_dimensional_object_reference;
mod unknown_0x2e;
mod unknown_0x34;
mod vertex_color;
mod vertex_color_reference;
mod zone_unknown;

use std::any::Any;
use std::marker::PhantomData;
use std::ops::Deref;

use nom::number::complete::le_i32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{StringReference, WResult};

pub use alternate_mesh::*;
pub use ambient_light::*;
pub use blit_sprite_definition::*;
pub use blit_sprite_reference::*;
pub use bsp_region::*;
pub use bsp_tree::*;
pub use camera::*;
pub use camera_reference::*;
pub use common::*;
pub use first::*;
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
pub use polygon_animation::*;
pub use polygon_animation_reference::*;
pub use region_flag::*;
pub use skeleton_track_set::*;
pub use skeleton_track_set_reference::*;
pub use sphere_list::*;
pub use sphere_list_def::*;
pub use texture::*;
pub use texture_images::*;
pub use texture_reference::*;
pub use two_dimensional_object::*;
pub use two_dimensional_object_reference::*;
pub use unknown_0x2e::*;
pub use unknown_0x34::*;
pub use vertex_color::*;
pub use vertex_color_reference::*;
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
    AlternateMesh(AlternateMeshFragment),
    AmbientLight(AmbientLightFragment),
    BlitSpriteDefinition(BlitSpriteDefinitionFragment),
    BlitSpriteReference(BlitSpriteReferenceFragment),
    BspRegion(BspRegionFragment),
    BspTree(BspTreeFragment),
    Camera(CameraFragment),
    CameraReference(CameraReferenceFragment),
    First(FirstFragment),
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
    PolygonAnimation(PolygonAnimationFragment),
    PolygonAnimationReference(PolygonAnimationReferenceFragment),
    RegionFlag(RegionFlagFragment),
    SkeletonTrackSet(SkeletonTrackSetFragment),
    SkeletonTrackSetReference(SkeletonTrackSetReferenceFragment),
    SphereList(SphereListFragment),
    SphereListDef(SphereListDefFragment),
    Texture(TextureFragment),
    TextureImages(TextureImagesFragment),
    TextureReference(TextureReferenceFragment),
    TwoDimensionalObject(TwoDimensionalObjectFragment),
    TwoDimensionalObjectReference(TwoDimensionalObjectReferenceFragment),
    Unknown0x34(Unknown0x34Fragment),
    Unknown0x2e(Unknown0x2eFragment),
    VertexColor(VertexColorFragment),
    VertexColorReference(VertexColorReferenceFragment),
    ZoneUnknown(ZoneUnknownFragment),
}

impl Deref for FragmentType {
    type Target = dyn Fragment;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::AlternateMesh(x) => x,
            Self::AmbientLight(x) => x,
            Self::BlitSpriteDefinition(x) => x,
            Self::BlitSpriteReference(x) => x,
            Self::BspRegion(x) => x,
            Self::BspTree(x) => x,
            Self::Camera(x) => x,
            Self::CameraReference(x) => x,
            Self::First(x) => x,
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
            Self::PolygonAnimation(x) => x,
            Self::PolygonAnimationReference(x) => x,
            Self::RegionFlag(x) => x,
            Self::SkeletonTrackSet(x) => x,
            Self::SkeletonTrackSetReference(x) => x,
            Self::SphereList(x) => x,
            Self::SphereListDef(x) => x,
            Self::Texture(x) => x,
            Self::TextureImages(x) => x,
            Self::TextureReference(x) => x,
            Self::TwoDimensionalObject(x) => x,
            Self::TwoDimensionalObjectReference(x) => x,
            Self::Unknown0x34(x) => x,
            Self::Unknown0x2e(x) => x,
            Self::VertexColor(x) => x,
            Self::VertexColorReference(x) => x,
            Self::ZoneUnknown(x) => x,
        }
    }
}
