use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use super::{ACTIVE_BLOCK_COLOR, INACTIVE_BLOCK_COLOR};
use crate::app::App;

pub fn draw_filter(f: &mut Frame, _app: &App, layout_chunk: Rect, active: bool) {
    let border_color = match active {
        true => ACTIVE_BLOCK_COLOR,
        false => INACTIVE_BLOCK_COLOR,
    };

    let paragraph = Paragraph::new(Line::from("Search")).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(paragraph, layout_chunk);
}
