use libeq_wld::parser::{fragments::*, FragmentType};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{ActiveBlock, App};
use crate::ui::{ACTIVE_BLOCK_COLOR, INACTIVE_BLOCK_COLOR};

fn draw_fragment<'a>(app: &'a App, fragment_type: &FragmentType) -> ListItem<'a> {
    let name = app
        .wld_doc
        .get_string(*fragment_type.name_ref())
        .map_or("".to_string(), |n| format!(" ({})", n));

    let (frag_type_name, color) = match fragment_type {
        FragmentType::AlternateMesh(_) => (
            AlternateMeshFragment::TYPE_NAME,
            Color::Rgb(0xad, 0xff, 0x2f),
        ),
        FragmentType::AmbientLight(_) => (
            AmbientLightFragment::TYPE_NAME,
            Color::Rgb(0xa0, 0x20, 0xf0),
        ),
        FragmentType::BspRegion(_) => (BspRegionFragment::TYPE_NAME, Color::Rgb(0x00, 0xff, 0xff)),
        FragmentType::BspTree(_) => (BspTreeFragment::TYPE_NAME, Color::Rgb(0x00, 0xfa, 0x9a)),
        FragmentType::Camera(_) => (CameraFragment::TYPE_NAME, Color::Rgb(0x48, 0x3d, 0x8b)),
        FragmentType::CameraReference(_) => (
            CameraReferenceFragment::TYPE_NAME,
            Color::Rgb(0xb2, 0x22, 0x22),
        ),
        FragmentType::First(_) => (FirstFragment::TYPE_NAME, Color::Rgb(0x7b, 0x68, 0xee)),
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
        FragmentType::Texture(_) => (TextureFragment::TYPE_NAME, Color::Rgb(0x2f, 0x4f, 0x4f)),
        FragmentType::TextureImages(_) => (
            TextureImagesFragment::TYPE_NAME,
            Color::Rgb(0xa9, 0xa9, 0xa9),
        ),
        FragmentType::TextureReference(_) => (
            TextureReferenceFragment::TYPE_NAME,
            Color::Rgb(0x8b, 0x45, 0x13),
        ),
        FragmentType::TwoDimensionalObject(_) => (
            TwoDimensionalObjectFragment::TYPE_NAME,
            Color::Rgb(0x00, 0x64, 0x00),
        ),
        FragmentType::TwoDimensionalObjectReference(_) => (
            TwoDimensionalObjectFragment::TYPE_NAME,
            Color::Rgb(0x80, 0x80, 0x00),
        ),
        FragmentType::VertexColor(_) => {
            (VertexColorFragment::TYPE_NAME, Color::Rgb(0xdd, 0xa0, 0xdd))
        }
        FragmentType::VertexColorReference(_) => (
            VertexColorReferenceFragment::TYPE_NAME,
            Color::Rgb(0xff, 0x14, 0x93),
        ),
        FragmentType::ZoneUnknown(_) => {
            (ZoneUnknownFragment::TYPE_NAME, Color::Rgb(0xb0, 0x30, 0x60))
        }
    };

    let lines = vec![Spans::from(vec![Span::styled(
        format!("{}{}", frag_type_name, name),
        Style::default().fg(color),
    )])];
    ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
}

pub fn draw_fragment_list<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let list_items: Vec<_> = app.wld_doc.iter().map(|f| draw_fragment(&app, f)).collect();

    draw_selectable_list(
        f,
        app,
        layout_chunk,
        &list_items,
        matches!(app.route.active_block, ActiveBlock::FragmentList),
        app.selected_fragment_idx,
    );
}

pub fn draw_selectable_list<B>(
    f: &mut Frame<B>,
    _app: &App,
    layout_chunk: Rect,
    items: &[ListItem],
    active: bool,
    selected_index: Option<usize>,
) where
    B: Backend,
{
    let mut state = ListState::default();
    state.select(selected_index);

    let border_color = match active {
        true => ACTIVE_BLOCK_COLOR,
        false => INACTIVE_BLOCK_COLOR,
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Fragments")
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, layout_chunk, &mut state);
}
