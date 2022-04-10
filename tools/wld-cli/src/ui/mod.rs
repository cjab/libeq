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
