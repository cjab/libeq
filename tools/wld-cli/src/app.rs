use std::error::Error;

use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::handlers::{handle_key_event, handle_mouse_event};
use crate::{event::Event, event::Events};
use libeq_wld::parser::WldDoc;

pub struct App {
    pub wld_doc: WldDoc,
    pub route: Route,
    pub selected_fragment_idx: Option<usize>,
    pub detail_scroll_pos: (u16, u16),
    pub detail_body_tab_idx: usize,
}

impl App {
    pub fn new(wld_doc: WldDoc) -> App {
        App {
            wld_doc,
            route: DEFAULT_ROUTE,
            selected_fragment_idx: None,
            detail_body_tab_idx: 0,
            detail_scroll_pos: (0, 0),
        }
    }

    pub fn handle_events(&mut self, events: &Events) -> Result<bool, Box<dyn Error>> {
        match events.next()? {
            Event::Input(input) => match input {
                event::Event::Key(key_event) if key_event.code == KeyCode::Char('q') => {
                    return Ok(false);
                }
                event::Event::Key(key_event)
                    if key_event.code == KeyCode::Char('c')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    return Ok(false);
                }
                event::Event::Key(key_event) => handle_key_event(key_event, self),
                event::Event::Mouse(mouse_event) => handle_mouse_event(mouse_event, self),
                _ => {}
            },
            Event::Tick => {}
        }
        Ok(true)
    }
}

pub enum ActiveBlock {
    FilterInput,
    FragmentList,
    FragmentDetails,
}

pub struct Route {
    pub id: RouteId,
    pub active_block: ActiveBlock,
}

pub enum RouteId {
    Main,
}

const DEFAULT_ROUTE: Route = Route {
    id: RouteId::Main,
    active_block: ActiveBlock::FragmentList,
};
