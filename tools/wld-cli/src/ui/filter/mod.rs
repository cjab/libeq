use tui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{ACTIVE_BLOCK_COLOR, INACTIVE_BLOCK_COLOR};
use crate::app::App;

pub fn draw_filter<B>(f: &mut Frame<B>, _app: &App, layout_chunk: Rect, active: bool)
where
    B: Backend,
{
    let border_color = match active {
        true => ACTIVE_BLOCK_COLOR,
        false => INACTIVE_BLOCK_COLOR,
    };

    let paragraph = Paragraph::new(Spans::from("Search")).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(paragraph, layout_chunk);
}
