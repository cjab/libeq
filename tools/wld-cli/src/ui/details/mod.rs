use ansi_to_tui::ansi_to_text;
use hexyl::{BorderStyle, Printer};
use libeq_wld::parser::{fragments, FragmentType};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
    Frame,
};

use super::{get_frag_name_and_color, ACTIVE_BLOCK_COLOR, INACTIVE_BLOCK_COLOR};
use crate::app::{ActiveBlock, App};

const TABLE_WIDTHS: [Constraint; 2] = [Constraint::Length(10), Constraint::Length(100)];
const NEWLINE: u8 = 10;

pub fn draw_fragment_details<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    if app.selected_fragment_idx.is_none() {
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)].as_ref())
        .split(layout_chunk);

    let fragment_idx = app.selected_fragment_idx.expect("No fragment selected");
    let fragment = app
        .wld_doc
        .at(fragment_idx)
        .expect("Invalid fragment selected");

    draw_fragment_header(f, app, layout[0], fragment_idx, fragment);
    draw_fragment_body(f, app, layout[1], fragment_idx, fragment);
}

pub fn draw_fragment_header<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) where
    B: Backend,
{
    let border_color = match app.route.active_block {
        ActiveBlock::FragmentDetails => ACTIVE_BLOCK_COLOR,
        _ => INACTIVE_BLOCK_COLOR,
    };
    let name = app
        .wld_doc
        .get_string(*fragment.name_ref())
        .map_or("".to_string(), |n| format!("{}", n));
    let (frag_type_name, frag_color) = get_frag_name_and_color(fragment);

    let table = Table::new(vec![
        Row::new(vec![Span::styled(
            format!("{}", frag_type_name),
            Style::default().fg(frag_color),
        )]),
        Row::new(vec![name]),
    ])
    .block(
        Block::default()
            .title(format!("Header - {}", fragment_idx + 1))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&[Constraint::Length(100), Constraint::Length(0)])
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_fragment_body<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) where
    B: Backend,
{
    let border_color = match app.route.active_block {
        ActiveBlock::FragmentDetails => ACTIVE_BLOCK_COLOR,
        _ => INACTIVE_BLOCK_COLOR,
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(layout_chunk);

    let tabs = Tabs::new(["Fields", "Raw"].iter().cloned().map(Spans::from).collect())
        .block(Block::default())
        .select(app.detail_body_tab_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, layout[0]);

    match app.detail_body_tab_idx {
        0 => {
            draw_fragment_fields(f, app, layout[1], fragment_idx, fragment);
        }
        _ => {
            draw_raw_fragment_data(f, app, layout[1], fragment_idx, fragment);
        }
    }
}

pub fn draw_raw_fragment_data<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) where
    B: Backend,
{
    let border_color = match app.route.active_block {
        ActiveBlock::FragmentDetails => ACTIVE_BLOCK_COLOR,
        _ => INACTIVE_BLOCK_COLOR,
    };

    let mut hex = vec![];
    let mut hex_printer = Printer::new(&mut hex, true, BorderStyle::Unicode, true);
    hex_printer
        .print_all(&fragment.into_bytes()[..])
        .expect("Error printing hex");

    let lines: Vec<u8> = hex
        .split(|c| *c == NEWLINE)
        .take(100)
        .intersperse(&[NEWLINE])
        .flatten()
        .map(|c| *c)
        .collect();

    let paragraph = Paragraph::new(ansi_to_text(lines).unwrap()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(paragraph, layout_chunk);
}

pub fn draw_fragment_fields<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) where
    B: Backend,
{
    let border_color = match app.route.active_block {
        ActiveBlock::FragmentDetails => ACTIVE_BLOCK_COLOR,
        _ => INACTIVE_BLOCK_COLOR,
    };

    match fragment {
        FragmentType::TextureImages(frag) => {
            draw_texture_images_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::Texture(frag) => {
            draw_texture_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::TextureReference(frag) => {
            draw_texture_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::Material(frag) => {
            draw_material_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::AmbientLight(frag) => {
            draw_ambient_light_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::LightSourceReference(frag) => {
            draw_light_source_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::RegionFlag(frag) => {
            draw_region_flag_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::ObjectLocation(frag) => {
            draw_object_location_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::CameraReference(frag) => {
            draw_camera_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::BspRegion(frag) => {
            draw_bsp_region_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::Model(frag) => {
            draw_model_reference_player_info_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::BspTree(frag) => {
            draw_bsp_tree_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::Camera(frag) => {
            draw_camera_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::LightSource(frag) => {
            draw_light_source_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::MaterialList(frag) => {
            draw_material_list_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::Mesh(frag) => {
            draw_mesh_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::VertexColorReference(frag) => {
            draw_vertex_color_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::VertexColor(frag) => {
            draw_vertex_color_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::LightInfo(frag) => {
            draw_light_info_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::MeshReference(frag) => {
            draw_mesh_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::MobSkeletonPieceTrack(frag) => {
            draw_mob_skeleton_piece_track_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::MobSkeletonPieceTrackReference(frag) => {
            draw_mob_skeleton_piece_track_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::SkeletonTrackSet(frag) => {
            draw_skeleton_track_set_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::TwoDimensionalObject(frag) => {
            draw_two_dimensional_object_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::Unknown0x34(frag) => {
            draw_unknown_0x34_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::Unknown0x2e(frag) => {
            draw_unknown_0x2e_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::BlitSpriteDefinition(frag) => {
            draw_blit_sprite_definition_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::BlitSpriteReference(frag) => {
            draw_blit_sprite_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::AlternateMesh(frag) => {
            draw_alternate_mesh_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::MeshAnimatedVertices(frag) => {
            draw_mesh_animated_vertices_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::MeshAnimatedVerticesReference(frag) => {
            draw_mesh_animated_vertices_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::PolygonAnimation(frag) => {
            draw_polygon_animation_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::PolygonAnimationReference(frag) => {
            draw_polygon_animation_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::First(frag) => {
            draw_first_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::SkeletonTrackSetReference(frag) => {
            draw_skeleton_track_set_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::TwoDimensionalObjectReference(frag) => {
            draw_two_dimensional_object_reference_fragment(f, app, layout_chunk, &frag);
        }
        FragmentType::ZoneUnknown(frag) => {
            draw_zone_unknown_fragment(f, app, layout_chunk, &frag);
        }
    }
}

pub fn draw_texture_images_fragment<B>(
    f: &mut Frame<B>,
    _app: &App,
    layout_chunk: Rect,
    fragment: &fragments::TextureImagesFragment,
) where
    B: Backend,
{
    let size = fragment.size1.to_string();
    let filenames = &fragment
        .entries
        .iter()
        .map(|e| e.file_name.clone())
        .collect::<Vec<_>>()
        .join("\n");

    let table = Table::new(vec![
        Row::new(vec!["Size1", &size]),
        Row::new(vec!["Filenames", &filenames]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_texture_fragment<B>(
    f: &mut Frame<B>,
    _app: &App,
    layout_chunk: Rect,
    fragment: &fragments::TextureFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags.0, fragment.flags.0);
    let frame_count = fragment.frame_count.to_string();
    let current_frame = format!("{:?}", fragment.current_frame);
    let sleep = format!("{:?}", fragment.sleep);
    let frame_references = &fragment
        .frame_references
        .iter()
        .map(|e| format!("{:?}", e))
        .collect::<Vec<_>>()
        .join("\n");

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Frame count", &frame_count]),
        Row::new(vec!["Current frame", &current_frame]),
        Row::new(vec!["Sleep", &sleep]),
        Row::new(vec!["Frame references", &frame_references]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_texture_reference_fragment<B>(
    f: &mut Frame<B>,
    _app: &App,
    layout_chunk: Rect,
    fragment: &fragments::TextureReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_material_fragment<B>(
    f: &mut Frame<B>,
    _app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MaterialFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let transparency_flags = format!(
        "0x{:x}  (b{:0>32b})",
        Into::<u32>::into(fragment.transparency_flags),
        Into::<u32>::into(fragment.transparency_flags),
    );
    let params2 = format!("0x{:x}  (b{:0>32b})", fragment.params2, fragment.params2);
    let mask_color_coord = match fragment.mask_color_coord {
        (p1, p2) => format!("(0x{:x}, 0x{:x})", p1 as u32, p2 as u32),
    };
    let reference = format!("{:?}", fragment.reference);
    let pair = match fragment.pair {
        Some((p1, p2)) => format!(
            "0x{:x} (b{:0>32b})    0x{:x} (b{:0>32b})",
            p1 as u32, p1 as u32, p2 as u32, p2 as u32
        ),
        None => "None".to_string(),
    };

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Transparency flags", &transparency_flags]),
        Row::new(vec!["Params2", &params2]),
        Row::new(vec!["Mask color coord", &mask_color_coord]),
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Pair", &pair]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_ambient_light_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::AmbientLightFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let region_count = format!("{}", fragment.region_count);
    let region_ids = format!("{:?}", fragment.regions);

    let table = Table::new(vec![
        Row::new(vec!["Light Source Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Region Count", &region_count]),
        Row::new(vec!["Region Ids", &region_ids]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_light_source_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::LightSourceReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);

    let table = Table::new(vec![
        Row::new(vec!["Light Source", &reference]),
        Row::new(vec!["Flags", &flags]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_region_flag_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::RegionFlagFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let region_count = format!("{}", fragment.region_count);
    let region_ids = format!("{:?}", fragment.regions);
    let user_data_size = format!("{:?}", fragment.user_data_size);
    let user_data = format!("{:?}", fragment.user_data);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Region Count", &region_count]),
        Row::new(vec!["Region Ids", &region_ids]),
        Row::new(vec!["User data size", &user_data_size]),
        Row::new(vec!["User data", &user_data]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_object_location_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::ObjectLocationFragment,
) where
    B: Backend,
{
    let actor_def_reference = format!("{:?}", fragment.actor_def_reference);
    let flags = format!("{:?}", fragment.flags);
    let sphere_reference = format!("{}", fragment.sphere_reference);
    let current_action = format!("{:?}", fragment.current_action);
    let location = format!("{:?}", fragment.location);
    let bounding_radius = format!("{:?}", fragment.bounding_radius);
    let scale_factor = format!("{:?}", fragment.scale_factor);
    let sound_name_reference = format!("{:?}", fragment.sound_name_reference);
    let unknown = format!("{}", fragment.unknown);

    let table = Table::new(vec![
        Row::new(vec!["ACTORDEF reference", &actor_def_reference]),
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["SPHERE reference", &sphere_reference]),
        Row::new(vec!["Current action", &current_action]),
        Row::new(vec!["Location", &location]),
        Row::new(vec!["Bounding radius", &bounding_radius]),
        Row::new(vec!["Scale factor", &scale_factor]),
        Row::new(vec!["Sound name ref", &sound_name_reference]),
        Row::new(vec!["Unknown", &unknown]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_camera_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::CameraReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_bsp_region_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::BspRegionFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let fragment1 = format!("{:?}", fragment.fragment1);
    let size1 = format!("{:?}", fragment.size1);
    let size2 = format!("{:?}", fragment.size2);
    let params1 = format!("{:?}", fragment.params1);
    let size3 = format!("{:?}", fragment.size3);
    let size4 = format!("{:?}", fragment.size4);
    let params2 = format!("{:?}", fragment.params2);
    let size5 = format!("{:?}", fragment.size5);
    let pvs_count = format!("{:?}", fragment.pvs_count);
    let data1 = format!("{:?}", fragment.data1);
    let data2 = format!("{:?}", fragment.data2);
    let data3 = format!("{:?}", fragment.data3);
    let data4 = format!("{:?}", fragment.data4);
    let data5 = format!("{:?}", fragment.data5);
    let pvs = format!("{:?}", fragment.pvs);
    let size7 = format!("{:?}", fragment.size7);
    let name7 = format!("{:?}", fragment.name7);
    let fragment2 = format!("{:?}", fragment.fragment2);
    let mesh_reference = format!("{:?}", fragment.mesh_reference);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Fragment1", &fragment1]),
        Row::new(vec!["Size1", &size1]),
        Row::new(vec!["Size2", &size2]),
        Row::new(vec!["Params1", &params1]),
        Row::new(vec!["Size3", &size3]),
        Row::new(vec!["Size4", &size4]),
        Row::new(vec!["Params2", &params2]),
        Row::new(vec!["Size5", &size5]),
        Row::new(vec!["PVS Count", &pvs_count]),
        Row::new(vec!["Data1", &data1]),
        Row::new(vec!["Data2", &data2]),
        Row::new(vec!["Data3", &data3]),
        Row::new(vec!["Data4", &data4]),
        Row::new(vec!["Data5", &data5]),
        Row::new(vec!["PVS", &pvs]),
        Row::new(vec!["Size7", &size7]),
        Row::new(vec!["Name7", &name7]),
        Row::new(vec!["Fragment2", &fragment2]),
        Row::new(vec!["Mesh Reference", &mesh_reference]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_model_reference_player_info_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::ModelFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);
    let callback_name_reference = format!("{:?}", fragment.callback_name_reference);
    let action_count = format!("{:?}", fragment.action_count);
    let fragment_reference_count = format!("{:?}", fragment.fragment_reference_count);
    let bounds_reference = format!("{:?}", fragment.bounds_reference);
    let current_action = format!("{:?}", fragment.current_action);
    let location = format!("{:?}", fragment.location);
    let actions = format!("{:?}", fragment.actions);
    let fragment_references = format!("{:?}", fragment.fragment_references);
    let unknown = format!("{:?}", fragment.unknown);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Callback name reference", &callback_name_reference]),
        Row::new(vec!["Action Count", &action_count]),
        Row::new(vec!["Fragment Ref Count", &fragment_reference_count]),
        Row::new(vec!["Bounds ref", &bounds_reference]),
        Row::new(vec!["Current Action", &current_action]),
        Row::new(vec!["Location", &location]),
        Row::new(vec!["Actions", &actions]),
        Row::new(vec!["Fragment references", &fragment_references]),
        Row::new(vec!["Unknown", &unknown]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_bsp_tree_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::BspTreeFragment,
) where
    B: Backend,
{
    let world_node_count = format!("{:?}", fragment.world_node_count);
    let world_nodes = format!("{:?}", fragment.world_nodes);

    let table = Table::new(vec![
        Row::new(vec!["World node count", &world_node_count]),
        Row::new(vec!["World nodes", &world_nodes]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_camera_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::CameraFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);
    let vertex_count = format!("{:?}", fragment.vertex_count);
    let bsp_node_count = format!("{:?}", fragment.bsp_node_count);
    let sphere_list_reference = format!("{:?}", fragment.sphere_list_reference);
    let center_offset = format!("{:?}", fragment.center_offset);
    let bounding_radius = format!("{:?}", fragment.bounding_radius);
    let vertices = format!("{:?}", fragment.vertices);
    let bsp_node_entries = format!("{:?}", fragment.bsp_nodes);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Vertex count", &vertex_count]),
        Row::new(vec!["Bsp node count", &bsp_node_count]),
        Row::new(vec!["Sphere list reference", &sphere_list_reference]),
        Row::new(vec!["Center offset", &center_offset]),
        Row::new(vec!["Bounding radius", &bounding_radius]),
        Row::new(vec!["Vertices", &vertices]),
        Row::new(vec!["Bsp nodes", &bsp_node_entries]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_light_source_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::LightSourceFragment,
) where
    B: Backend,
{
    let flags = format!(
        "0x{:x}  (b{:0>32b})",
        fragment.flags.to_u32(),
        fragment.flags.to_u32()
    );
    let frame_count = format!("{:?}", fragment.frame_count);
    let current_frame = format!("{:?}", fragment.current_frame);
    let sleep = format!("{:?}", fragment.sleep);
    let light_levels = format!("{:?}", fragment.light_levels);
    let colors = format!("{:?}", fragment.colors);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Frame count", &frame_count]),
        Row::new(vec!["Current frame", &current_frame]),
        Row::new(vec!["Sleep", &sleep]),
        Row::new(vec!["Light levels", &light_levels]),
        Row::new(vec!["Colors", &colors]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_material_list_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MaterialListFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let size1 = format!("{:?}", fragment.size1);
    let fragments = format!("{:?}", fragment.fragments);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Size1", &size1]),
        Row::new(vec!["Fragments", &fragments]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_mesh_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MeshFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let material_list_ref = format!("{:?}", fragment.material_list_ref);
    let animation_ref = format!("{:?}", fragment.animation_ref);
    let fragment3 = format!("{:?}", fragment.fragment3);
    let fragment4 = format!("{:?}", fragment.fragment4);
    let center = format!("{:?}", fragment.center);
    let params2 = format!("{:?}", fragment.params2);
    let max_distance = format!("{:?}", fragment.max_distance);
    let min = format!("{:?}", fragment.min);
    let max = format!("{:?}", fragment.max);
    let position_count = format!("{:?}", fragment.position_count);
    let texture_coordinate_count = format!("{:?}", fragment.texture_coordinate_count);
    let normal_count = format!("{:?}", fragment.normal_count);
    let color_count = format!("{:?}", fragment.color_count);
    let polygon_count = format!("{:?}", fragment.polygon_count);
    let vertex_piece_count = format!("{:?}", fragment.vertex_piece_count);
    let polygon_material_count = format!("{:?}", fragment.polygon_material_count);
    let vertex_material_count = format!("{:?}", fragment.vertex_material_count);
    let size9 = format!("{:?}", fragment.size9);
    let scale = format!("{:?}", fragment.scale);
    let positions = format!("{:?}", fragment.positions);
    let texture_coordinates = format!("{:?}", fragment.texture_coordinates);
    let vertex_normals = format!("{:?}", fragment.vertex_normals);
    let vertex_colors = format!("{:?}", fragment.vertex_colors);
    let polygons = format!("{:?}", fragment.polygons);
    let vertex_pieces = format!("{:?}", fragment.vertex_pieces);
    let polygon_materials = format!("{:?}", fragment.polygon_materials);
    let vertex_materials = format!("{:?}", fragment.vertex_materials);
    //let data9 = format!("{:?}", fragment.data9);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Material List Ref", &material_list_ref]),
        Row::new(vec!["Animation Ref", &animation_ref]),
        Row::new(vec!["Fragment 3", &fragment3]),
        Row::new(vec!["Fragment 4", &fragment4]),
        Row::new(vec!["Center", &center]),
        Row::new(vec!["Params2", &params2]),
        Row::new(vec!["Max Distance", &max_distance]),
        Row::new(vec!["Min", &min]),
        Row::new(vec!["Max", &max]),
        Row::new(vec!["Position Count", &position_count]),
        Row::new(vec!["Texture Coord Count", &texture_coordinate_count]),
        Row::new(vec!["Normal Count", &normal_count]),
        Row::new(vec!["Color Count", &color_count]),
        Row::new(vec!["Polygon Count", &polygon_count]),
        Row::new(vec!["Vertex Piece Count", &vertex_piece_count]),
        Row::new(vec!["Polygon Material Count", &polygon_material_count]),
        Row::new(vec!["Vertex Material Count", &vertex_material_count]),
        Row::new(vec!["Size9", &size9]),
        Row::new(vec!["Scale", &scale]),
        Row::new(vec!["Positions", &positions]),
        Row::new(vec!["Texture Coordinates", &texture_coordinates]),
        Row::new(vec!["Vertex Normals", &vertex_normals]),
        Row::new(vec!["Vertex Colors", &vertex_colors]),
        Row::new(vec!["Polygons", &polygons]),
        Row::new(vec!["Vertex Pieces", &vertex_pieces]),
        Row::new(vec!["Polygon Materials", &polygon_materials]),
        Row::new(vec!["Vertex Materials", &vertex_materials]),
        //Row::new(vec!["Data9", &data9]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_vertex_color_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::VertexColorReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_vertex_color_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::VertexColorFragment,
) where
    B: Backend,
{
    let data1 = format!("{:?}", fragment.data1);
    let vertex_color_count = format!("{:?}", fragment.vertex_color_count);
    let data2 = format!("{:?}", fragment.data2);
    let data3 = format!("{:?}", fragment.data3);
    let data4 = format!("{:?}", fragment.data4);
    let vertex_colors = format!("{:?}", fragment.vertex_colors);

    let table = Table::new(vec![
        Row::new(vec!["Data1", &data1]),
        Row::new(vec!["Vertex Color Count", &vertex_color_count]),
        Row::new(vec!["Data2", &data2]),
        Row::new(vec!["Data3", &data3]),
        Row::new(vec!["Data4", &data4]),
        Row::new(vec!["Vertex Colors", &vertex_colors]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_light_info_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::LightInfoFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("{:?}", fragment.flags);
    let x = format!("{:?}", fragment.x);
    let y = format!("{:?}", fragment.y);
    let z = format!("{:?}", fragment.z);
    let radius = format!("{:?}", fragment.radius);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["X", &x]),
        Row::new(vec!["Y", &y]),
        Row::new(vec!["Z", &z]),
        Row::new(vec!["Radius", &radius]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_mesh_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MeshReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let params = format!("{:?}", fragment.params);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Params", &params]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_mob_skeleton_piece_track_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MobSkeletonPieceTrackFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let frame_count = format!("{:?}", fragment.frame_count);
    let frame_transforms = format!("{:?}", fragment.frame_transforms);
    let data2 = format!("{:?}", fragment.data2);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Frame count", &frame_count]),
        Row::new(vec!["Frame transforms", &frame_transforms]),
        Row::new(vec!["Data2", &data2]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_mob_skeleton_piece_track_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MobSkeletonPieceTrackReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("{:?}", fragment.flags);
    let sleep = format!("{:?}", fragment.sleep);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Sleep", &sleep]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_skeleton_track_set_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::SkeletonTrackSetFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);
    let num_dags = format!("{:?}", fragment.num_dags);
    let collision_volume_reference = format!("{:?}", fragment.collision_volume_reference);
    let center_offset = format!("{:?}", fragment.center_offset);
    let bounding_radius = format!("{:?}", fragment.bounding_radius);
    let dags = format!("{:?}", fragment.dags);
    let size2 = format!("{:?}", fragment.size2);
    let fragment3 = format!("{:?}", fragment.fragment3);
    let data3 = format!("{:?}", fragment.data3);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Num dags", &num_dags]),
        Row::new(vec!["Collision volume ref", &collision_volume_reference]),
        Row::new(vec!["Center offset", &center_offset]),
        Row::new(vec!["Bounding radius", &bounding_radius]),
        Row::new(vec!["Dags", &dags]),
        Row::new(vec!["Size2", &size2]),
        Row::new(vec!["Fragment3", &fragment3]),
        Row::new(vec!["Data3", &data3]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_two_dimensional_object_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::TwoDimensionalObjectFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);
    let num_frames = format!("{:?}", fragment.num_frames);
    let num_pitches = format!("{:?}", fragment.num_pitches);
    let sprite_size = format!("{:?}", fragment.sprite_size);
    let sphere_fragment = format!("{:?}", fragment.sphere_fragment);
    let depth_scale = format!("{:?}", fragment.depth_scale);
    let center_offset = format!("{:?}", fragment.center_offset);
    let bounding_radius = format!("{:?}", fragment.bounding_radius);
    let current_frame = format!("{:?}", fragment.current_frame);
    let sleep = format!("{:?}", fragment.sleep);
    let pitches = format!("{:?}", fragment.pitches);
    let render_method = format!("{:?}", fragment.render_method);
    let render_flags = format!("{:?}", fragment.render_info.flags);
    let pen = format!("{:?}", fragment.render_info.pen);
    let brightness = format!("{:?}", fragment.render_info.brightness);
    let scaled_ambient = format!("{:?}", fragment.render_info.scaled_ambient);
    let uv_info = format!("{:?}", fragment.render_info.uv_info);
    let uv_map = format!("{:?}", fragment.render_info.uv_map);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Num Frames", &num_frames]),
        Row::new(vec!["Num Pitches", &num_pitches]),
        Row::new(vec!["Sprite Size", &sprite_size]),
        Row::new(vec!["Sphere Fragment", &sphere_fragment]),
        Row::new(vec!["Depth Scale", &depth_scale]),
        Row::new(vec!["Center Offset", &center_offset]),
        Row::new(vec!["Bounding radius", &bounding_radius]),
        Row::new(vec!["Current frame", &current_frame]),
        Row::new(vec!["Sleep", &sleep]),
        Row::new(vec!["Pitches", &pitches]),
        Row::new(vec!["Render Method", &render_method]),
        Row::new(vec!["Render Flags", &render_flags]),
        Row::new(vec!["Pen", &pen]),
        Row::new(vec!["Brightness", &brightness]),
        Row::new(vec!["Scaled Ambient", &scaled_ambient]),
        Row::new(vec!["UV Info", &uv_info]),
        Row::new(vec!["UV Map", &uv_map]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_unknown_0x34_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::Unknown0x34Fragment,
) where
    B: Backend,
{
    let unknown_1 = format!("{}", fragment.unknown_1);
    let unknown_2 = format!("{}", fragment.unknown_2);
    let unknown_3 = format!("{}", fragment.unknown_3);
    let unknown_4 = format!("{}", fragment.unknown_4);
    let unknown_5 = format!("{}", fragment.unknown_5);
    let unknown_6 = format!("{}", fragment.unknown_6);
    let unknown_7 = format!("{}", fragment.unknown_7);
    let unknown_8 = format!("{}", fragment.unknown_8);
    let unknown_9 = format!("{}", fragment.unknown_9);
    let unknown_10 = format!("{}", fragment.unknown_10);
    let unknown_11 = format!("{}", fragment.unknown_11);
    let unknown_12 = format!("{}", fragment.unknown_12);
    let unknown_13 = format!("{}", fragment.unknown_13);
    let unknown_14 = format!("{}", fragment.unknown_14);
    let unknown_15 = format!("{}", fragment.unknown_15);
    let unknown_16 = format!("{}", fragment.unknown_16);
    let unknown_17 = format!("{}", fragment.unknown_17);
    let unknown_18 = format!("{}", fragment.unknown_18);
    let unknown_19 = format!("{}", fragment.unknown_19);
    let unknown_20 = format!("{}", fragment.unknown_20);
    let unknown_21 = format!("{}", fragment.unknown_21);

    let table = Table::new(vec![
        Row::new(vec!["Unknown 1", &unknown_1]),
        Row::new(vec!["Unknown 2", &unknown_2]),
        Row::new(vec!["Unknown 3", &unknown_3]),
        Row::new(vec!["Unknown 4", &unknown_4]),
        Row::new(vec!["Unknown 5", &unknown_5]),
        Row::new(vec!["Unknown 6", &unknown_6]),
        Row::new(vec!["Unknown 7", &unknown_7]),
        Row::new(vec!["Unknown 8", &unknown_8]),
        Row::new(vec!["Unknown 9", &unknown_9]),
        Row::new(vec!["Unknown 10", &unknown_10]),
        Row::new(vec!["Unknown 11", &unknown_11]),
        Row::new(vec!["Unknown 12", &unknown_12]),
        Row::new(vec!["Unknown 13", &unknown_13]),
        Row::new(vec!["Unknown 14", &unknown_14]),
        Row::new(vec!["Unknown 15", &unknown_15]),
        Row::new(vec!["Unknown 16", &unknown_16]),
        Row::new(vec!["Unknown 17", &unknown_17]),
        Row::new(vec!["Unknown 18", &unknown_18]),
        Row::new(vec!["Unknown 19", &unknown_19]),
        Row::new(vec!["Unknown 20", &unknown_20]),
        Row::new(vec!["Unknown 21", &unknown_21]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_unknown_0x2e_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::Unknown0x2eFragment,
) where
    B: Backend,
{
    let table = Table::new(vec![])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .widths(&TABLE_WIDTHS)
        .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_blit_sprite_definition_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::BlitSpriteDefinitionFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);
    let reference = format!("{:?}", fragment.blit_sprite_reference);
    let unknown = format!("{}", fragment.unknown);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Blit sprite ref", &reference]),
        Row::new(vec!["Unknown", &unknown]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_blit_sprite_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::BlitSpriteReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.blit_sprite_reference);
    let unknown = format!("{}", fragment.unknown);

    let table = Table::new(vec![
        Row::new(vec!["Blit sprite ref", &reference]),
        Row::new(vec!["Unknown", &unknown]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

/// TODO: Incomplete
pub fn draw_alternate_mesh_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::AlternateMeshFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);

    let table = Table::new(vec![Row::new(vec!["flags", &flags])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .widths(&TABLE_WIDTHS)
        .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

/// TODO: Incomplete
pub fn draw_mesh_animated_vertices_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MeshAnimatedVerticesFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);

    let table = Table::new(vec![Row::new(vec!["flags", &flags])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .widths(&TABLE_WIDTHS)
        .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_mesh_animated_vertices_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::MeshAnimatedVerticesReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("{:?}", fragment.flags);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

/// TODO: Incomplete
pub fn draw_polygon_animation_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::PolygonAnimationFragment,
) where
    B: Backend,
{
    let flags = format!("{:?}", fragment.flags);

    let table = Table::new(vec![Row::new(vec!["Flags", &flags])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .widths(&TABLE_WIDTHS)
        .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_polygon_animation_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::PolygonAnimationReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("{:?}", fragment.flags);
    let scale_factor = format!("{:?}", fragment.scale_factor);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Scale factor", &scale_factor]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_first_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::FirstFragment,
) where
    B: Backend,
{
    let table = Table::new(vec![])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .widths(&TABLE_WIDTHS)
        .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_skeleton_track_set_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::SkeletonTrackSetReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let params1 = format!("{:?}", fragment.params1);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Params 1", &params1]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_zone_unknown_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::ZoneUnknownFragment,
) where
    B: Backend,
{
    let radius = format!("{:?}", fragment.radius);

    let table = Table::new(vec![Row::new(vec!["Radius", &radius])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default()),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .widths(&TABLE_WIDTHS)
        .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_two_dimensional_object_reference_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::TwoDimensionalObjectReferenceFragment,
) where
    B: Backend,
{
    let reference = format!("{:?}", fragment.reference);
    let flags = format!("{:?}", fragment.flags);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default()),
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .widths(&TABLE_WIDTHS)
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}
