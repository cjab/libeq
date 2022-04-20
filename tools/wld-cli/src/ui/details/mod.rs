use ansi_to_tui::ansi_to_text;
use eq_wld::parser::{fragments, FragmentType};
use hexyl::{BorderStyle, Printer};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
    Frame,
};

use super::{ACTIVE_BLOCK_COLOR, INACTIVE_BLOCK_COLOR};
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
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
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

    let table = Table::new(vec![
        Row::new(vec!["Size", "--"]),
        Row::new(vec!["Type", "--"]),
        Row::new(vec!["Name Ref", "--"]),
    ])
    .block(
        Block::default()
            .title(format!("Header - 0x{:x} ({})", fragment_idx, fragment_idx))
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
    .widths(&TABLE_WIDTHS)
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
        .print_all(&fragment.serialize()[..])
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
        _ => {}
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
    let params1 = format!("0x{:x}  (b{:0>32b})", fragment.params1, fragment.params1);
    let params2 = format!("0x{:x}  (b{:0>32b})", fragment.params2, fragment.params2);
    let params3 = match fragment.params3 {
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
        Row::new(vec!["Params1", &params1]),
        Row::new(vec!["Params2", &params2]),
        Row::new(vec!["Params3", &params3]),
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
    let size2 = format!("{:?}", fragment.size2);
    let data2 = format!("{:?}", fragment.data2);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Region Count", &region_count]),
        Row::new(vec!["Region Ids", &region_ids]),
        Row::new(vec!["Size2", &size2]),
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

pub fn draw_object_location_fragment<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    fragment: &fragments::ObjectLocationFragment,
) where
    B: Backend,
{
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let fragment1 = format!("{}", fragment.fragment1);
    let x = format!("{}", fragment.x);
    let y = format!("{}", fragment.y);
    let z = format!("{}", fragment.z);
    let rotate_z = format!("{}", fragment.rotate_z);
    let rotate_y = format!("{}", fragment.rotate_y);
    let rotate_x = format!("{}", fragment.rotate_x);
    let params1 = format!("{}", fragment.params1);
    let scale_y = format!("{}", fragment.scale_y);
    let scale_x = format!("{}", fragment.scale_x);
    let fragment2 = format!("{}", fragment.fragment2);
    let params2 = format!("{}", fragment.params2);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Fragment1", &fragment1]),
        Row::new(vec!["X", &x]),
        Row::new(vec!["Y", &y]),
        Row::new(vec!["Z", &z]),
        Row::new(vec!["Rotate Z", &rotate_z]),
        Row::new(vec!["Rotate Y", &rotate_y]),
        Row::new(vec!["Rotate X", &rotate_x]),
        Row::new(vec!["Params1", &params1]),
        Row::new(vec!["Scale Y", &scale_y]),
        Row::new(vec!["Scale X", &scale_x]),
        Row::new(vec!["Fragment2", &fragment2]),
        Row::new(vec!["Params2", &params2]),
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
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let name_fragment = format!("{:?}", fragment.name_fragment);
    let unknown_params2_count = format!("{:?}", fragment.unknown_params2_count);
    let fragment_count = format!("{:?}", fragment.fragment_count);
    let unknown_fragment = format!("{:?}", fragment.unknown_fragment);
    let unknown_params1 = format!("{:?}", fragment.unknown_params1);
    let unknown_params2 = format!("{:?}", fragment.unknown_params2);
    let unknown_data_count = format!("{:?}", fragment.unknown_data_count);
    let unknown_data = format!("{:?}", fragment.unknown_data);
    let fragments = format!("{:?}", fragment.fragments);
    let name_size = format!("{:?}", fragment.name_size);
    let name = format!("{:?}", fragment.name);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Name Fragment", &name_fragment]),
        Row::new(vec!["Params2 Count", &unknown_params2_count]),
        Row::new(vec!["Fragment Count", &fragment_count]),
        Row::new(vec!["Fragment", &unknown_fragment]),
        Row::new(vec!["Params1", &unknown_params1]),
        Row::new(vec!["Params2", &unknown_params2]),
        Row::new(vec!["Data Count", &unknown_data_count]),
        Row::new(vec!["Data", &unknown_data]),
        Row::new(vec!["Fragments", &fragments]),
        Row::new(vec!["Name Size", &name_size]),
        Row::new(vec!["Name", &name]),
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
    let size1 = format!("{:?}", fragment.size1);
    let entries = format!("{:?}", fragment.entries);

    let table = Table::new(vec![
        Row::new(vec!["Size1", &size1]),
        Row::new(vec!["Entries", &entries]),
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
    let params0 = format!("{:?}", fragment.params0);
    let params1 = format!("{:?}", fragment.params1);
    let params2 = format!("{:?}", fragment.params2);
    let params3 = format!("{:?}", fragment.params3);
    let params4 = format!("{:?}", fragment.params4);
    let params5 = format!("{:?}", fragment.params5);
    let params6 = format!("{:?}", fragment.params6);
    let params7 = format!("{:?}", fragment.params7);
    let params8 = format!("{:?}", fragment.params8);
    let params9 = format!("{:?}", fragment.params9);
    let params10 = format!("{:?}", fragment.params10);
    let params11 = format!("{:?}", fragment.params11);
    let params12 = format!("{:?}", fragment.params12);
    let params13 = format!("{:?}", fragment.params13);
    let params14 = format!("{:?}", fragment.params14);
    let params15 = format!("{:?}", fragment.params15);
    let params16 = format!("{:?}", fragment.params16);
    let params17 = format!("{:?}", fragment.params17);
    let params18 = format!("{:?}", fragment.params18);
    let params19 = format!("{:?}", fragment.params19);
    let params20 = format!("{:?}", fragment.params20);
    let params21 = format!("{:?}", fragment.params21);
    let params22 = format!("{:?}", fragment.params22);
    let params23 = format!("{:?}", fragment.params23);
    let params24 = format!("{:?}", fragment.params24);
    let params25 = format!("{:?}", fragment.params25);

    let table = Table::new(vec![
        Row::new(vec!["Params0", &params0]),
        Row::new(vec!["Params1", &params1]),
        Row::new(vec!["Params2", &params2]),
        Row::new(vec!["Params3", &params3]),
        Row::new(vec!["Params4", &params4]),
        Row::new(vec!["Params5", &params5]),
        Row::new(vec!["Params6", &params6]),
        Row::new(vec!["Params7", &params7]),
        Row::new(vec!["Params8", &params8]),
        Row::new(vec!["Params9", &params9]),
        Row::new(vec!["Params10", &params10]),
        Row::new(vec!["Params11", &params11]),
        Row::new(vec!["Params12", &params12]),
        Row::new(vec!["Params13", &params13]),
        Row::new(vec!["Params14", &params14]),
        Row::new(vec!["Params15", &params15]),
        Row::new(vec!["Params16", &params16]),
        Row::new(vec!["Params17", &params17]),
        Row::new(vec!["Params18", &params18]),
        Row::new(vec!["Params19", &params19]),
        Row::new(vec!["Params20", &params20]),
        Row::new(vec!["Params21", &params21]),
        Row::new(vec!["Params22", &params22]),
        Row::new(vec!["Params23", &params23]),
        Row::new(vec!["Params24", &params24]),
        Row::new(vec!["Params25", &params25]),
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
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let params2 = format!("{:?}", fragment.params2);
    let params3a = format!("{:?}", fragment.params3a);
    let params3b = format!("{:?}", fragment.params3b);
    let params4 = format!("{:?}", fragment.params4);
    let red = format!("{:?}", fragment.red);
    let green = format!("{:?}", fragment.green);
    let blue = format!("{:?}", fragment.blue);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Params2", &params2]),
        Row::new(vec!["Params3a", &params3a]),
        Row::new(vec!["Params3b", &params3b]),
        Row::new(vec!["Params4", &params4]),
        Row::new(vec!["Red", &red]),
        Row::new(vec!["Green", &green]),
        Row::new(vec!["Blue", &blue]),
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
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let x = format!("{:?}", fragment.x);
    let y = format!("{:?}", fragment.y);
    let z = format!("{:?}", fragment.z);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["X", &x]),
        Row::new(vec!["Y", &y]),
        Row::new(vec!["Z", &z]),
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
    let size = format!("{:?}", fragment.size);
    let rotate_denominator = format!("{:?}", fragment.rotate_denominator);
    let rotate_x_numerator = format!("{:?}", fragment.rotate_x_numerator);
    let rotate_y_numerator = format!("{:?}", fragment.rotate_y_numerator);
    let rotate_z_numerator = format!("{:?}", fragment.rotate_z_numerator);
    let shift_x_numerator = format!("{:?}", fragment.shift_x_numerator);
    let shift_y_numerator = format!("{:?}", fragment.shift_y_numerator);
    let shift_z_numerator = format!("{:?}", fragment.shift_z_numerator);
    let shift_denominator = format!("{:?}", fragment.shift_denominator);
    let data2 = format!("{:?}", fragment.data2);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Size", &size]),
        Row::new(vec!["Rotate Denominator", &rotate_denominator]),
        Row::new(vec!["Rotate X Numerator", &rotate_x_numerator]),
        Row::new(vec!["Rotate Y Numerator", &rotate_y_numerator]),
        Row::new(vec!["Rotate Z Numerator", &rotate_z_numerator]),
        Row::new(vec!["Shift X Numerator", &shift_x_numerator]),
        Row::new(vec!["Shift Y Numerator", &shift_y_numerator]),
        Row::new(vec!["Shift Z Numerator", &shift_z_numerator]),
        Row::new(vec!["Shift Denominator", &shift_denominator]),
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
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let params1 = format!("{:?}", fragment.params1);

    let table = Table::new(vec![
        Row::new(vec!["Reference", &reference]),
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Params1", &params1]),
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
    let flags = format!("0x{:x}  (b{:0>32b})", fragment.flags, fragment.flags);
    let entry_count = format!("{:?}", fragment.entry_count);
    let fragment_ref = format!("{:?}", fragment.fragment);
    let unknown_params1 = format!("{:?}", fragment.unknown_params1);
    let unknown_params2 = format!("{:?}", fragment.unknown_params2);
    let entries = format!("{:?}", fragment.entries);
    let size2 = format!("{:?}", fragment.size2);
    let fragment3 = format!("{:?}", fragment.fragment3);
    let data3 = format!("{:?}", fragment.data3);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Entry Count", &entry_count]),
        Row::new(vec!["Fragment Ref", &fragment_ref]),
        Row::new(vec!["Params1", &unknown_params1]),
        Row::new(vec!["Params2", &unknown_params2]),
        Row::new(vec!["Entries", &entries]),
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
    let render_flags = format!("{:?}", fragment.render_flags);
    let pen = format!("{:?}", fragment.pen);
    let brightness = format!("{:?}", fragment.brightness);
    let scaled_ambient = format!("{:?}", fragment.scaled_ambient);
    let params7_fragment = format!("{:?}", fragment.params7_fragment);
    let uv_info = format!("{:?}", fragment.uv_info);
    let params7_size = format!("{:?}", fragment.params7_size);
    let params7_data = format!("{:?}", fragment.params7_data);

    let table = Table::new(vec![
        Row::new(vec!["Flags", &flags]),
        Row::new(vec!["Num Frames", &num_frames]),
        Row::new(vec!["Num Pitches", &num_pitches]),
        Row::new(vec!["Sprite Size", &sprite_size]),
        Row::new(vec!["Sphere Fragment", &sphere_fragment]),
        Row::new(vec!["Depth Scale", &depth_scale]),
        Row::new(vec!["Center Offset", &center_offset]),
        Row::new(vec!["Params 4", &bounding_radius]),
        Row::new(vec!["Params 5", &current_frame]),
        Row::new(vec!["Sleep", &sleep]),
        Row::new(vec!["Pitches", &pitches]),
        Row::new(vec!["Render Method", &render_method]),
        Row::new(vec!["Render Flags", &render_flags]),
        Row::new(vec!["Pen", &pen]),
        Row::new(vec!["Brightness", &brightness]),
        Row::new(vec!["Scaled Ambient", &scaled_ambient]),
        Row::new(vec!["Params7 Fragment", &params7_fragment]),
        Row::new(vec!["UV Info", &uv_info]),
        Row::new(vec!["Params7 Size", &params7_size]),
        Row::new(vec!["Params7 Data", &params7_data]),
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
