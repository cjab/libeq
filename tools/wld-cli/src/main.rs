#![feature(iter_intersperse)]

mod app;
mod event;
mod handlers;
mod ui;

use std::fs::File;
use std::io::{prelude::*, Read};
use std::path::Path;
use std::{error::Error, io};

use clap::{arg, Command};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use crate::{app::App, event::Events};
use libeq_wld::parser::{self, WldDoc};

fn cli() -> Command<'static> {
    Command::new("wld-cli")
        .version("0.1.0")
        .author("Chad Jablonski <chad@jablonski.xyz>")
        .about("Work with data from EverQuest .wld files")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("explore")
                .about("Display a TUI interface listing all fragments in the file")
                .arg_required_else_help(true)
                .arg(arg!(<WLD_FILE> "The wld file to explore")),
        )
        .subcommand(
            Command::new("extract")
                .about("Extract fragments from the wld file")
                .arg_required_else_help(true)
                .arg(arg!(<WLD_FILE> "The source wld file"))
                .arg(arg!(<DESTINATION> "The target destination")),
        )
        .subcommand(
            Command::new("stats")
                .about("Display stats about the wld file")
                .arg_required_else_help(true)
                .arg(arg!(<WLD_FILE> "The wld file")),
        )
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    match cli().get_matches().subcommand() {
        Some(("explore", sub_matches)) => {
            let wld_file = sub_matches.value_of("WLD_FILE").expect("required");
            println!("EXPLORE: {:?}", wld_file);
            explore(wld_file)?;
        }
        Some(("extract", sub_matches)) => {
            let wld_file = sub_matches.value_of("WLD_FILE").expect("required");
            let destination = sub_matches.value_of("DESTINATION").expect("required");
            println!("EXTRACT: {:?} -> {:?}", wld_file, destination);
            extract(wld_file, destination)?;
        }
        Some(("stats", sub_matches)) => {
            let wld_file = sub_matches.value_of("WLD_FILE").expect("required");
            println!("STATS: {:?}", wld_file);
            stats(wld_file)?;
        }
        Some(_) => (),
        None => (),
    }

    Ok(())
}

fn explore(wld_filename: &str) -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let wld_data = read_wld_file(wld_filename).expect("Could not read wld file");
    let (_, wld_doc) = parser::WldDoc::parse(&wld_data).expect("Could not read wld file");
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

fn extract(wld_filename: &str, destination: &str) -> Result<(), Box<dyn Error>> {
    let wld_data = read_wld_file(wld_filename).expect("Could not read wld file");
    let (_, raw_fragments) =
        WldDoc::dump_raw_fragments(&wld_data).expect("Could not read wld file");

    for (i, fragment_header) in raw_fragments.iter().enumerate() {
        let filename = format!("{:04}-{:#04x}.frag", i, fragment_header.fragment_type);
        let dest = Path::new(destination).join(filename);
        let mut file = File::create(&dest).expect(&format!("Failed to create file: {:?}", dest));
        file.write_all(fragment_header.field_data).unwrap();
    }

    Ok(())
}

fn stats(wld_filename: &str) -> Result<(), Box<dyn Error>> {
    //    let wld_doc = open_wld_file(wld_filename);
    let (_, wld_doc) = parser::WldDoc::parse(&read_wld_file(wld_filename)?).unwrap();
    //let stats = wld_doc
    //    .fragment_iter()
    //    .fold(HashMap::new(), |mut map, fragment| {
    //        map.entry(header.fragment_type)
    //            .or_insert_with(|| Vec::new())
    //            .push(header);
    //        map
    //    });
    //let mut sorted_keys: Vec<_> = stats.keys().collect();
    //sorted_keys.sort();
    //for k in sorted_keys {
    //    println!("0x{:02x?}: {}", k, stats[k].len());
    //}

    Ok(())
}

fn read_wld_file(filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = File::open(filename)?;
    let mut wld_data = Vec::new();
    file.read_to_end(&mut wld_data)?;
    Ok(wld_data)
}
