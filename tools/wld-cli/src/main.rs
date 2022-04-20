#![feature(iter_intersperse)]

mod app;
mod event;
mod handlers;
mod ui;

use std::fs::File;
use std::io::Read;
use std::{error::Error, io};

use clap::{App as Clapp, Arg};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use crate::{app::App, event::Events};
use eq_wld::parser;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let matches = Clapp::new("wld-cli")
        .version("0.1.0")
        .author("Chad Jablonski <chad@jablonski.xyz>")
        .about("Extract data from EverQuest .wld files")
        .arg(
            Arg::with_name("WLD_FILE")
                .help("The s3d archive to extract from")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("stats")
                .short("s")
                .long("stats")
                .help("Get file stats")
                .takes_value(false),
        )
        .get_matches();

    let wld_filename = matches.value_of("WLD_FILE").unwrap();

    let mut file = File::open(wld_filename)?;
    let mut wld_data = Vec::new();
    file.read_to_end(&mut wld_data)?;

    let (_, wld_doc) = parser::WldDoc::parse(&wld_data).unwrap();

    let show_stats = matches.is_present("stats");

    //if show_stats {
    //    let stats = wld_doc
    //        .fragments
    //        .iter()
    //        .fold(HashMap::new(), |mut map, header| {
    //            map.entry(header.fragment_type)
    //                .or_insert_with(|| Vec::new())
    //                .push(header);
    //            map
    //        });
    //    let mut sorted_keys: Vec<_> = stats.keys().collect();
    //    sorted_keys.sort();
    //    for k in sorted_keys {
    //        println!("0x{:02x?}: {}", k, stats[k].len());
    //    }
    //    return Ok(());
    //}

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut app = App::new(wld_doc);

    loop {
        terminal.draw(|f| {
            ui::draw_main_layout(f, &app);
        })?;

        if !app.handle_events(&events).unwrap() {
            break;
        }
    }

    Ok(())
}
