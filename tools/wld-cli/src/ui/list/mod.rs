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
use crate::ui::{get_frag_name_and_color, ACTIVE_BLOCK_COLOR, INACTIVE_BLOCK_COLOR};

fn draw_fragment<'a>(app: &'a App, idx: usize, fragment_type: &FragmentType) -> ListItem<'a> {
    let name = app
        .wld_doc
        .get_string(*fragment_type.name_ref())
        .map_or("".to_string(), |n| format!(" ({})", n));

    let (frag_type_name, color) = get_frag_name_and_color(fragment_type);

    let lines = vec![Spans::from(vec![
        Span::styled(format!("{:>5} ", idx), Style::default()),
        Span::styled(
            format!("{}{}", frag_type_name, name),
            Style::default().fg(color),
        ),
    ])];
    ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
}

pub fn draw_fragment_list<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let list_items: Vec<_> = app
        .wld_doc
        .iter()
        .enumerate()
        .map(|(idx, f)| draw_fragment(&app, idx, f))
        .collect();

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
