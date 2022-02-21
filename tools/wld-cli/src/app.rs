use std::error::Error;

use termion::event::Key;

use crate::handlers::handle_app;
use crate::{event::Event, event::Events};
use eq_wld::parser::WldDoc;

pub struct App<'a> {
    pub wld_doc: WldDoc<'a>,
    pub route: Route,
    pub filter_input: String,
    pub selected_fragment_idx: Option<usize>,
    pub detail_body_tab_idx: usize,
}

impl<'a> App<'a> {
    pub fn new(wld_doc: WldDoc) -> App {
        App {
            wld_doc,
            route: DEFAULT_ROUTE,
            selected_fragment_idx: None,
            detail_body_tab_idx: 0,
            filter_input: String::default(),
        }
    }

    pub fn handle_events(&mut self, events: &Events) -> Result<bool, Box<dyn Error>> {
        match events.next()? {
            // Quit
            Event::Input(Key::Char('q')) => return Ok(false),
            Event::Input(Key::Ctrl('c')) => return Ok(false),
            Event::Input(input) => handle_app(input, self),
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
