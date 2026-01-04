use std::cmp;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{ActiveBlock, App, RouteId};

const HALF_PAGE_STEP: usize = 25;

pub fn handle_app(event: KeyEvent, app: &mut App) {
    match event.code {
        KeyCode::Char('/') => {
            app.route.active_block = ActiveBlock::FilterInput;
        }
        // Move left
        KeyCode::Left | KeyCode::Char('h') => match app.route.id {
            RouteId::Main => {
                app.route.active_block = ActiveBlock::FragmentList;
            }
        },
        // Move right
        KeyCode::Right | KeyCode::Char('l') => match app.route.id {
            RouteId::Main => {
                app.route.active_block = ActiveBlock::FragmentDetails;
            }
        },
        // Tab
        KeyCode::Tab => match app.route.id {
            RouteId::Main => {
                app.detail_scroll_pos = (0, 0);
                app.detail_body_tab_idx = wrap_idx(app.detail_body_tab_idx as i32 + 1, 3);
            }
        },
        // Tab back
        KeyCode::BackTab => match app.route.id {
            RouteId::Main => {
                app.detail_scroll_pos = (0, 0);
                app.detail_body_tab_idx = wrap_idx(app.detail_body_tab_idx as i32 - 1, 3);
            }
        },
        // Move down
        KeyCode::Down | KeyCode::Char('j') => match app.route.id {
            RouteId::Main => match app.route.active_block {
                ActiveBlock::FragmentList => {
                    let fragment_count = app.wld_doc.fragment_count();
                    app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                        Some(i) => cmp::min(i + 1, fragment_count - 1),
                        None => 0,
                    });
                    app.detail_scroll_pos = (0, 0);
                }
                ActiveBlock::FragmentDetails => {
                    app.detail_scroll_pos.0 += 1;
                }
                ActiveBlock::FilterInput => {}
            },
        },
        // Move up
        KeyCode::Up | KeyCode::Char('k') => match app.route.id {
            RouteId::Main => match app.route.active_block {
                ActiveBlock::FragmentList => {
                    app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                        Some(i) => cmp::max(i as i32 - 1, 0 as i32) as usize,
                        None => 0,
                    });
                    app.detail_scroll_pos = (0, 0);
                }
                ActiveBlock::FragmentDetails => {
                    app.detail_scroll_pos.0 =
                        cmp::max(0i32, app.detail_scroll_pos.0 as i32 - 1) as u16;
                }
                ActiveBlock::FilterInput => {}
            },
        },
        // Half page down
        KeyCode::Char('d') if event.modifiers.contains(KeyModifiers::CONTROL) => match app.route.id
        {
            RouteId::Main => match app.route.active_block {
                ActiveBlock::FragmentList => {
                    let fragment_count = app.wld_doc.fragment_count();
                    app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                        Some(i) => cmp::min(i + HALF_PAGE_STEP, fragment_count - 1),
                        None => 0,
                    });
                    app.detail_scroll_pos = (0, 0);
                }
                ActiveBlock::FragmentDetails => {
                    app.detail_scroll_pos.0 += HALF_PAGE_STEP as u16;
                }
                ActiveBlock::FilterInput => {}
            },
        },
        // Half page up
        KeyCode::Char('u') if event.modifiers.contains(KeyModifiers::CONTROL) => match app.route.id
        {
            RouteId::Main => match app.route.active_block {
                ActiveBlock::FragmentList => {
                    app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                        Some(i) => cmp::max(i as i32 - HALF_PAGE_STEP as i32, 0i32) as usize,
                        None => 0,
                    });
                    app.detail_scroll_pos = (0, 0);
                }
                ActiveBlock::FragmentDetails => {
                    app.detail_scroll_pos.0 =
                        cmp::max(0i32, app.detail_scroll_pos.0 as i32 - HALF_PAGE_STEP as i32)
                            as u16;
                }
                ActiveBlock::FilterInput => {}
            },
        },
        KeyCode::Char('G') => match app.route.id {
            RouteId::Main => match app.route.active_block {
                ActiveBlock::FragmentList => {
                    let fragment_count = app.wld_doc.fragment_count();
                    app.selected_fragment_idx = Some(fragment_count - 1);
                    app.detail_scroll_pos = (0, 0);
                }
                ActiveBlock::FragmentDetails => {}
                ActiveBlock::FilterInput => {}
            },
        },
        _ => {}
    }
}

fn wrap_idx(idx: i32, idx_max: i32) -> usize {
    (((idx % idx_max) + idx_max) % idx_max).abs() as usize
}
