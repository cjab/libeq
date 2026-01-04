use ansi_to_tui::IntoText as _;
use hexyl::{BorderStyle, Printer};
use libeq_wld::parser::FragmentType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs},
};

use super::{ACTIVE_BLOCK_COLOR, INACTIVE_BLOCK_COLOR, get_frag_name_and_color};
use crate::app::{ActiveBlock, App};

const TABLE_WIDTHS: [Constraint; 2] = [Constraint::Length(100), Constraint::Length(0)];
const NEWLINE: u8 = 10;

pub fn draw_fragment_details(f: &mut Frame, app: &App, layout_chunk: Rect) {
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

pub fn draw_fragment_header(
    f: &mut Frame,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) {
    let name = app
        .wld_doc
        .get_string(*fragment.name_ref())
        .map_or("".to_string(), |n| format!("{}", n));
    let (frag_type_name, frag_color) = get_frag_name_and_color(fragment);

    let table = Table::new(
        vec![
            Row::new(vec![Span::styled(
                format!("{}", frag_type_name),
                Style::default().fg(frag_color),
            )]),
            Row::new(vec![name]),
        ],
        TABLE_WIDTHS,
    )
    .block(
        Block::default()
            .title(format!("Header - {}", fragment_idx + 1))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(INACTIVE_BLOCK_COLOR)),
    )
    .style(Style::default().fg(Color::White))
    .row_highlight_style(
        Style::default()
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .column_spacing(1);

    f.render_widget(table, layout_chunk);
}

pub fn draw_fragment_body(
    f: &mut Frame,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) {
    let border_color = match app.route.active_block {
        ActiveBlock::FragmentDetails => ACTIVE_BLOCK_COLOR,
        _ => INACTIVE_BLOCK_COLOR,
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(layout_chunk);

    let tabs = Tabs::new(["Fields", "JSON", "Raw"])
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
        1 => {
            draw_json_fragment_data(f, app, layout[1], fragment_idx, fragment);
        }
        _ => {
            draw_raw_fragment_data(f, app, layout[1], fragment_idx, fragment);
        }
    }
}

pub fn draw_raw_fragment_data(
    f: &mut Frame,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) {
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
        .collect::<Vec<_>>()
        .join(&[NEWLINE][..]);

    let paragraph = Paragraph::new(lines.into_text().unwrap()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(paragraph, layout_chunk);
}

pub fn draw_json_fragment_data(
    f: &mut Frame,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) {
    let border_color = match app.route.active_block {
        ActiveBlock::FragmentDetails => ACTIVE_BLOCK_COLOR,
        _ => INACTIVE_BLOCK_COLOR,
    };

    let json = serde_json::to_string_pretty(&fragment).expect("Could not serialize to json");

    let fields = Paragraph::new(json)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().fg(Color::White))
        .scroll(app.detail_scroll_pos);

    f.render_widget(fields, layout_chunk);
}

pub fn draw_fragment_fields(
    f: &mut Frame,
    app: &App,
    layout_chunk: Rect,
    fragment_idx: usize,
    fragment: &FragmentType,
) {
    let border_color = match app.route.active_block {
        ActiveBlock::FragmentDetails => ACTIVE_BLOCK_COLOR,
        _ => INACTIVE_BLOCK_COLOR,
    };

    let fields = Paragraph::new(format!("{:#?}", fragment))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().fg(Color::White))
        .scroll(app.detail_scroll_pos);

    f.render_widget(fields, layout_chunk);
}
