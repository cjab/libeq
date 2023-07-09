mod details;
mod filter;
mod list;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    Frame,
};

use crate::app::{ActiveBlock, App};
use details::draw_fragment_details;
use filter::draw_filter;
use list::draw_fragment_list;

use libeq_wld::parser::{fragments::*, FragmentType};

const ACTIVE_BLOCK_COLOR: Color = Color::Yellow;
const INACTIVE_BLOCK_COLOR: Color = Color::White;

pub fn draw_main_layout<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

    draw_filter(
        f,
        app,
        layout[0],
        matches!(app.route.active_block, ActiveBlock::FilterInput),
    );
    draw_content(f, app, layout[1]);
}

pub fn draw_content<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(layout_chunk);

    draw_fragment_list(f, app, layout[0]);
    draw_fragment_details(f, app, layout[1]);
}

pub fn get_frag_name_and_color(fragment_type: &FragmentType) -> (&'static str, Color) {
    match fragment_type {
        FragmentType::DmSpriteDef(_) => (
            DmSpriteDef::TYPE_NAME,
            Color::Rgb(0xad, 0xff, 0x2f),
        ),
        FragmentType::AmbientLight(_) => (
            AmbientLightFragment::TYPE_NAME,
            Color::Rgb(0xa0, 0x20, 0xf0),
        ),
        FragmentType::BlitSpriteDef(_) => (
            BlitSpriteDef::TYPE_NAME,
            Color::Rgb(0x0f, 0xff, 0xff),
        ),
        FragmentType::BlitSprite(_) => (
            BlitSprite::TYPE_NAME,
            Color::Rgb(0x0f, 0x2f, 0xff),
        ),
        FragmentType::BspRegion(_) => (BspRegionFragment::TYPE_NAME, Color::Rgb(0x00, 0xff, 0xff)),
        FragmentType::BspTree(_) => (BspTreeFragment::TYPE_NAME, Color::Rgb(0x00, 0xfa, 0x9a)),
        FragmentType::Camera(_) => (CameraFragment::TYPE_NAME, Color::Rgb(0x48, 0x3d, 0x8b)),
        FragmentType::CameraReference(_) => (
            CameraReferenceFragment::TYPE_NAME,
            Color::Rgb(0xb2, 0x22, 0x22),
        ),
        FragmentType::First(_) => (FirstFragment::TYPE_NAME, Color::Rgb(0x7b, 0x68, 0xee)),
        FragmentType::FourDSprite(_) => {
            (FourDSpriteFragment::TYPE_NAME, Color::Rgb(0xcc, 0x66, 0x66))
        }
        FragmentType::FourDSpriteDef(_) => (
            FourDSpriteDefFragment::TYPE_NAME,
            Color::Rgb(0xee, 0x99, 0x44),
        ),
        FragmentType::LightInfo(_) => (LightInfoFragment::TYPE_NAME, Color::Rgb(0x00, 0xbf, 0xff)),
        FragmentType::LightSource(_) => {
            (LightSourceFragment::TYPE_NAME, Color::Rgb(0xff, 0xff, 0x00))
        }
        FragmentType::LightSourceReference(_) => (
            LightSourceReferenceFragment::TYPE_NAME,
            Color::Rgb(0x00, 0xff, 0x00),
        ),
        FragmentType::Material(_) => (MaterialFragment::TYPE_NAME, Color::Rgb(0xf0, 0xe6, 0x8c)),
        FragmentType::MaterialList(_) => (
            MaterialListFragment::TYPE_NAME,
            Color::Rgb(0x64, 0x95, 0xed),
        ),
        FragmentType::Mesh(_) => (MeshFragment::TYPE_NAME, Color::Rgb(0xaf, 0xee, 0xee)),
        FragmentType::MeshAnimatedVertices(_) => (
            MeshAnimatedVerticesFragment::TYPE_NAME,
            Color::Rgb(0xff, 0xe4, 0xc4),
        ),
        FragmentType::MeshAnimatedVerticesReference(_) => (
            MeshAnimatedVerticesReferenceFragment::TYPE_NAME,
            Color::Rgb(0xff, 0x00, 0xff),
        ),
        FragmentType::MeshReference(_) => (
            MeshReferenceFragment::TYPE_NAME,
            Color::Rgb(0xff, 0x7f, 0x50),
        ),
        FragmentType::MobSkeletonPieceTrack(_) => (
            MobSkeletonPieceTrackFragment::TYPE_NAME,
            Color::Rgb(0x00, 0x00, 0x8b),
        ),
        FragmentType::MobSkeletonPieceTrackReference(_) => (
            MobSkeletonPieceTrackReferenceFragment::TYPE_NAME,
            Color::Rgb(0x32, 0xcd, 0x32),
        ),
        FragmentType::Model(_) => (ModelFragment::TYPE_NAME, Color::Rgb(0xda, 0xa5, 0x20)),
        FragmentType::ObjectLocation(_) => (
            ObjectLocationFragment::TYPE_NAME,
            Color::Rgb(0x8b, 0x00, 0x8b),
        ),
        FragmentType::ParticleSprite(_) => (
            ParticleSpriteFragment::TYPE_NAME,
            Color::Rgb(0x26, 0x59, 0x70),
        ),
        FragmentType::ParticleSpriteDef(_) => (
            ParticleSpriteDefFragment::TYPE_NAME,
            Color::Rgb(0x3c, 0x88, 0xab),
        ),
        FragmentType::ParticleCloudDef(_) => (
            ParticleCloudDefFragment::TYPE_NAME,
            Color::Rgb(0x80, 0x50, 0x05),
        ),
        FragmentType::PaletteFile(_) => {
            (PaletteFileFragment::TYPE_NAME, Color::Rgb(0x6a, 0x7f, 0xb5))
        }
        FragmentType::PolygonAnimation(_) => (
            PolygonAnimationFragment::TYPE_NAME,
            Color::Rgb(0xff, 0x45, 0x00),
        ),
        FragmentType::PolygonAnimationReference(_) => (
            PolygonAnimationReferenceFragment::TYPE_NAME,
            Color::Rgb(0xff, 0x8c, 0x00),
        ),
        FragmentType::RegionFlag(_) => {
            (RegionFlagFragment::TYPE_NAME, Color::Rgb(0x00, 0x00, 0xff))
        }
        FragmentType::SkeletonTrackSet(_) => (
            SkeletonTrackSetFragment::TYPE_NAME,
            Color::Rgb(0x3c, 0xb3, 0x71),
        ),
        FragmentType::SkeletonTrackSetReference(_) => (
            SkeletonTrackSetReferenceFragment::TYPE_NAME,
            Color::Rgb(0x00, 0x8b, 0x8b),
        ),
        FragmentType::SphereList(_) => {
            (SphereListFragment::TYPE_NAME, Color::Rgb(0x3c, 0xb3, 0x71))
        }
        FragmentType::SphereListDef(_) => (
            SphereListDefFragment::TYPE_NAME,
            Color::Rgb(0x00, 0x8b, 0x8b),
        ),
        FragmentType::SimpleSpriteDef(_) => (SimpleSpriteDef::TYPE_NAME, Color::Rgb(0x2f, 0x4f, 0x4f)),
        FragmentType::BmInfo(_) => (
            BmInfo::TYPE_NAME,
            Color::Rgb(0xa9, 0xa9, 0xa9),
        ),
        FragmentType::TextureImagesRtk(_) => (
            TextureImagesRtkFragment::TYPE_NAME,
            Color::Rgb(0xa9, 0xa9, 0xa9),
        ),
        FragmentType::SimpleSprite(_) => (
            SimpleSprite::TYPE_NAME,
            Color::Rgb(0x8b, 0x45, 0x13),
        ),
        FragmentType::Sprite2DDef(_) => (
            Sprite2DDef::TYPE_NAME,
            Color::Rgb(0x00, 0x64, 0x00),
        ),
        FragmentType::Sprite2D(_) => (
            Sprite2D::TYPE_NAME,
            Color::Rgb(0x80, 0x80, 0x00),
        ),
        FragmentType::Unknown0x2e(_) => {
            (Unknown0x2eFragment::TYPE_NAME, Color::Rgb(0x80, 0x50, 0x05))
        }
        FragmentType::VertexColor(_) => {
            (VertexColorFragment::TYPE_NAME, Color::Rgb(0xdd, 0xa0, 0xdd))
        }
        FragmentType::VertexColorReference(_) => (
            VertexColorReferenceFragment::TYPE_NAME,
            Color::Rgb(0xff, 0x14, 0x93),
        ),
        FragmentType::WorldVertices(_) => (
            WorldVerticesFragment::TYPE_NAME,
            Color::Rgb(0x59, 0x48, 0x78),
        ),
        FragmentType::ZoneUnknown(_) => {
            (ZoneUnknownFragment::TYPE_NAME, Color::Rgb(0xb0, 0x30, 0x60))
        }
    }
}
