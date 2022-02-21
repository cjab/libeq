use std::cmp;

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

use crate::app::{ActiveBlock, App, Route, RouteId};

const HALF_PAGE_STEP: usize = 25;

pub fn handle_app(key: Key, app: &mut App) {
    match key {
        Key::Char('/') => {
            app.route.active_block = ActiveBlock::FilterInput;
        }
        // Move left
        Key::Left | Key::Char('h') => match app.route.id {
            RouteId::Main => {
                app.route.active_block = ActiveBlock::FragmentList;
            }
        },
        // Move right
        Key::Right | Key::Char('l') => match app.route.id {
            RouteId::Main => {
                app.route.active_block = ActiveBlock::FragmentDetails;
            }
        },
        // Tab
        Key::Char('\t') => match app.route.id {
            RouteId::Main => {
                app.detail_body_tab_idx = (app.detail_body_tab_idx + 1) % 2;
            }
        },
        // Tab back
        Key::BackTab => match app.route.id {
            RouteId::Main => {
                app.detail_body_tab_idx = ((app.detail_body_tab_idx as i32 - 1).abs() % 2) as usize;
            }
        },
        // Move down
        Key::Down | Key::Char('j') => match app.route.id {
            RouteId::Main => {
                let fragment_header_len = app.wld_doc.fragments.len();
                app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                    Some(i) => cmp::min(i + 1, fragment_header_len - 1),
                    None => 0,
                });
            }
        },
        // Move up
        Key::Up | Key::Char('k') => match app.route.id {
            RouteId::Main => {
                let fragment_header_len = app.wld_doc.fragments.len();
                app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                    Some(i) => cmp::max(i as i32 - 1, 0 as i32) as usize,
                    None => 0,
                });
            }
        },
        // Half page down
        Key::Ctrl('d') => match app.route.id {
            RouteId::Main => {
                let fragment_header_len = app.wld_doc.fragments.len();
                app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                    Some(i) => cmp::min(i + HALF_PAGE_STEP, fragment_header_len - 1),
                    None => 0,
                });
            }
        },
        // Half page up
        Key::Ctrl('u') => match app.route.id {
            RouteId::Main => {
                app.selected_fragment_idx = Some(match app.selected_fragment_idx {
                    Some(i) => cmp::max(i as i32 - HALF_PAGE_STEP as i32, 0i32) as usize,
                    None => 0,
                });
            }
        },
        Key::Char('G') => match app.route.id {
            RouteId::Main => {
                let fragment_header_len = app.wld_doc.fragments.len();
                app.selected_fragment_idx = Some(fragment_header_len - 1);
            }
        },
        _ => {}
    }
}
