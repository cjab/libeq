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
            AmbientLight::TYPE_NAME,
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
        FragmentType::Region(_) => (Region::TYPE_NAME, Color::Rgb(0x00, 0xff, 0xff)),
        FragmentType::WorldTree(_) => (WorldTree::TYPE_NAME, Color::Rgb(0x00, 0xfa, 0x9a)),
        FragmentType::Sprite3DDef(_) => (Sprite3DDef::TYPE_NAME, Color::Rgb(0x48, 0x3d, 0x8b)),
        FragmentType::Sprite3D(_) => (
            Sprite3D::TYPE_NAME,
            Color::Rgb(0xb2, 0x22, 0x22),
        ),
        FragmentType::GlobalAmbientLightDef(_) => (GlobalAmbientLightDef::TYPE_NAME, Color::Rgb(0x7b, 0x68, 0xee)),
        FragmentType::Sprite4D(_) => {
            (Sprite4D::TYPE_NAME, Color::Rgb(0xcc, 0x66, 0x66))
        }
        FragmentType::Sprite4DDef(_) => (
            Sprite4DDef::TYPE_NAME,
            Color::Rgb(0xee, 0x99, 0x44),
        ),
        FragmentType::PointLight(_) => (PointLight::TYPE_NAME, Color::Rgb(0x00, 0xbf, 0xff)),
        FragmentType::LightDef(_) => {
            (LightDef::TYPE_NAME, Color::Rgb(0xff, 0xff, 0x00))
        }
        FragmentType::Light(_) => (
            Light::TYPE_NAME,
            Color::Rgb(0x00, 0xff, 0x00),
        ),
        FragmentType::MaterialDef(_) => (MaterialDef::TYPE_NAME, Color::Rgb(0xf0, 0xe6, 0x8c)),
        FragmentType::MaterialPalette(_) => (
            MaterialPalette::TYPE_NAME,
            Color::Rgb(0x64, 0x95, 0xed),
        ),
        FragmentType::DmSpriteDef2(_) => (DmSpriteDef2::TYPE_NAME, Color::Rgb(0xaf, 0xee, 0xee)),
        FragmentType::DmTrackDef2(_) => (
            DmTrackDef2::TYPE_NAME,
            Color::Rgb(0xff, 0xe4, 0xc4),
        ),
        FragmentType::DmTrack(_) => (
            DmTrack::TYPE_NAME,
            Color::Rgb(0xff, 0x00, 0xff),
        ),
        FragmentType::DmSprite(_) => (
            DmSprite::TYPE_NAME,
            Color::Rgb(0xff, 0x7f, 0x50),
        ),
        FragmentType::TrackDef(_) => (
            TrackDef::TYPE_NAME,
            Color::Rgb(0x00, 0x00, 0x8b),
        ),
        FragmentType::Track(_) => (
            Track::TYPE_NAME,
            Color::Rgb(0x32, 0xcd, 0x32),
        ),
        FragmentType::ActorDef(_) => (ActorDef::TYPE_NAME, Color::Rgb(0xda, 0xa5, 0x20)),
        FragmentType::Actor(_) => (
            Actor::TYPE_NAME,
            Color::Rgb(0x8b, 0x00, 0x8b),
        ),
        FragmentType::ParticleSprite(_) => (
            ParticleSprite::TYPE_NAME,
            Color::Rgb(0x26, 0x59, 0x70),
        ),
        FragmentType::ParticleSpriteDef(_) => (
            ParticleSpriteDef::TYPE_NAME,
            Color::Rgb(0x3c, 0x88, 0xab),
        ),
        FragmentType::ParticleCloudDef(_) => (
            ParticleCloudDef::TYPE_NAME,
            Color::Rgb(0x80, 0x50, 0x05),
        ),
        FragmentType::DefaultPaletteFile(_) => {
            (DefaultPaletteFile::TYPE_NAME, Color::Rgb(0x6a, 0x7f, 0xb5))
        }
        FragmentType::PolyhedronDef(_) => (
            PolyhedronDef::TYPE_NAME,
            Color::Rgb(0xff, 0x45, 0x00),
        ),
        FragmentType::Polyhedron(_) => (
            Polyhedron::TYPE_NAME,
            Color::Rgb(0xff, 0x8c, 0x00),
        ),
        FragmentType::Zone(_) => {
            (Zone::TYPE_NAME, Color::Rgb(0x00, 0x00, 0xff))
        }
        FragmentType::HierarchicalSpriteDef(_) => (
            HierarchicalSpriteDef::TYPE_NAME,
            Color::Rgb(0x3c, 0xb3, 0x71),
        ),
        FragmentType::HierarchicalSprite(_) => (
            HierarchicalSprite::TYPE_NAME,
            Color::Rgb(0x00, 0x8b, 0x8b),
        ),
        FragmentType::SphereList(_) => {
            (SphereList::TYPE_NAME, Color::Rgb(0x3c, 0xb3, 0x71))
        }
        FragmentType::SphereListDef(_) => (
            SphereListDef::TYPE_NAME,
            Color::Rgb(0x00, 0x8b, 0x8b),
        ),
        FragmentType::SimpleSpriteDef(_) => (SimpleSpriteDef::TYPE_NAME, Color::Rgb(0x2f, 0x4f, 0x4f)),
        FragmentType::BmInfo(_) => (
            BmInfo::TYPE_NAME,
            Color::Rgb(0xa9, 0xa9, 0xa9),
        ),
        FragmentType::BmInfoRtk(_) => (
            BmInfoRtk::TYPE_NAME,
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
        FragmentType::DmTrackDef(_) => {
            (DmTrackDef::TYPE_NAME, Color::Rgb(0x80, 0x50, 0x05))
        }
        FragmentType::DmRGBTrackDef(_) => {
            (DmRGBTrackDef::TYPE_NAME, Color::Rgb(0xdd, 0xa0, 0xdd))
        }
        FragmentType::DmRGBTrack(_) => (
            DmRGBTrack::TYPE_NAME,
            Color::Rgb(0xff, 0x14, 0x93),
        ),
        FragmentType::WorldVertices(_) => (
            WorldVerticesFragment::TYPE_NAME,
            Color::Rgb(0x59, 0x48, 0x78),
        ),
        FragmentType::Sphere(_) => {
            (Sphere::TYPE_NAME, Color::Rgb(0xb0, 0x30, 0x60))
        }
    }
}
