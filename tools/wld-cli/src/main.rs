#![feature(iter_intersperse)]

mod app;
mod event;
mod handlers;
mod ui;

use std::fs::{self, File};
use std::io::{prelude::*, Read};
use std::path::Path;
use std::{error::Error, io};

use clap::{arg, value_parser, Command, ValueEnum};
use colorful::Color;
use colorful::Colorful;
use hexyl::Printer;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use crate::{app::App, event::Events};
use libeq_wld::parser::{self, WldDoc, WldDocError};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    Raw,
    Json,
    Ron,
}

fn cli() -> Command<'static> {
    Command::new("wld-cli")
        .version("0.1.0")
        .author("Chad Jablonski <chad@jablonski.xyz>")
        .about("Work with data from EverQuest .wld files")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("explore")
                .about("Display a TUI interface listing all fragments in the file")
                .arg(arg!(<WLD_FILE> "The wld file to explore").required(true)),
        )
        .subcommand(
            Command::new("extract")
                .about("Extract fragments from the wld file")
                .arg(arg!(-f --format <FORMAT> "Format to extract to").value_parser(value_parser!(Format)).default_value("raw").required(false))
                .arg(arg!(<WLD_FILE> "The source wld file").required(true))
                .arg(arg!(<DESTINATION> "The target destination").required(true))
        )
        .subcommand(
            Command::new("create")
                .about("Create a wld file from a source directory of fragments")
                .arg(arg!(-f --format <FORMAT> "Format to extract to").value_parser(value_parser!(Format)).default_value("raw").required(false))
                .arg(arg!(<SOURCE> "The source directory containing a header, strings, and fragment files").required(true))
                .arg(arg!(<WLD_FILE> "The destination .wld file").required(true)),
        )
        .subcommand(
            Command::new("stats")
                .about("Display stats about the wld file")
                .arg(arg!(<WLD_FILE> "The wld file").required(true)),
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
            let format = sub_matches.get_one::<Format>("format").expect("required");
            println!(
                "EXTRACT: {:?} -> {:?} -- FORMAT {:?}",
                wld_file, destination, format
            );
            extract(wld_file, destination, format);
        }
        Some(("create", sub_matches)) => {
            let source = sub_matches.value_of("SOURCE").expect("required");
            let wld_file = sub_matches.value_of("WLD_FILE").expect("required");
            let format = sub_matches.get_one::<Format>("format").expect("required");
            println!("CREATE: {:?} -> {:?}", source, wld_file);
            create(source, wld_file, format);
        }
        Some(("stats", sub_matches)) => {
            let wld_file = sub_matches.value_of("WLD_FILE").expect("required");
            stats(wld_file)?;
        }
        Some(_) => (),
        None => (),
    }

    Ok(())
}

fn print_error(error: &WldDocError) -> Result<(), std::io::Error> {
    let mut out = io::stdout();
    match error {
        WldDocError::Parse { message, .. } => {
            write!(out, "{}", message)?;
        }
        WldDocError::ParseFragment {
            index,
            offset,
            header,
            message,
        } => {
            write!(out, "\n{}\n", "Failed Fragment".color(Color::Red))?;
            write!(out, "{}", message.clone().color(Color::LightPink1))?;
            write!(
                out,
                "\n{} 0x{:02x}, {} {}\n",
                "type:".color(Color::Grey54),
                header.fragment_type,
                "index:".color(Color::Grey54),
                index
            )?;
            let hex_offset = format!("0x{:02x}", offset).color(Color::DarkSeaGreen2);
            let dec_offset = format!("{}", offset).color(Color::DarkSeaGreen2);
            write!(
                out,
                "encountered at body offset: {} ({})\n",
                hex_offset, dec_offset
            )?;
            write!(out, "Dumping fragment body...\n")?;
            let mut hex_printer = Printer::new(&mut out, true, hexyl::BorderStyle::Unicode, true);
            hex_printer.print_all(header.field_data).unwrap();
        }
        WldDocError::UnknownFragment { index, header } => {
            write!(out, "\n{}\n", "Unknown Fragment".color(Color::Yellow))?;
            write!(
                out,
                "{} 0x{:02x}, {} {}\n",
                "type:".color(Color::Grey54),
                header.fragment_type,
                "index:".color(Color::Grey54),
                index
            )?;
            write!(out, "Dumping fragment body...\n")?;
            let mut hex_printer = Printer::new(&mut out, true, hexyl::BorderStyle::Unicode, true);
            hex_printer.print_all(header.field_data).unwrap();
        }
    }
    Ok(())
}

fn explore(wld_filename: &str) -> Result<(), Box<dyn Error>> {
    let wld_data = read_wld_file(wld_filename).expect("Could not read wld file");
    let wld_doc = parser::WldDoc::parse(&wld_data)
        .map_err(|e| {
            for error in e.iter() {
                print_error(error).unwrap();
            }
        })
        .expect("Could not read wld file");

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

fn extract(wld_filename: &str, destination: &str, format: &Format) {
    let wld_data = read_wld_file(wld_filename).expect("Could not read wld file");
    let wld_doc = parser::WldDoc::parse(&wld_data)
        .map_err(|e| {
            for error in e.iter() {
                print_error(error).unwrap();
            }
        })
        .expect("Could not read wld file");
    match format {
        Format::Raw => extract_raw(wld_filename, destination),
        Format::Json => {
            let out = fs::File::create(destination).expect("Could not create destination file");
            serde_json::to_writer_pretty(out, &wld_doc).expect("Could not serialize to json")
        }
        Format::Ron => {
            let out = fs::File::create(destination).expect("Could not create destination file");
            ron::ser::to_writer_pretty(out, &wld_doc, ron::ser::PrettyConfig::new())
                .expect("Could not serialize to json")
        }
    }
}

fn extract_raw(wld_filename: &str, destination: &str) {
    fs::create_dir_all(&destination).expect(&format!(
        "Could not create destination directory: {}",
        destination
    ));

    let wld_data = read_wld_file(wld_filename).expect("Could not read wld file");
    let (_, raw_fragments) =
        WldDoc::dump_raw_fragments(&wld_data).expect("Could not read wld file");
    let wld = parser::WldDoc::parse(&wld_data)
        .map_err(|e| {
            for error in e.iter() {
                print_error(error).unwrap();
            }
        })
        .expect("Could not read wld file");

    let header_path = Path::new(destination).join("0000--header.bin");
    let mut file = File::create(&header_path).expect(&format!("Failed to create header file"));
    file.write_all(&wld.header_bytes()).unwrap();

    let strings_path = Path::new(destination).join("0000--strings.bin");
    let mut file = File::create(&strings_path).expect(&format!("Failed to create strings file"));
    file.write_all(&wld.strings_bytes()).unwrap();

    for (i, fragment_header) in raw_fragments.iter().enumerate() {
        let filename = format!("{:04}-{:#04x}.frag", i, fragment_header.fragment_type);
        let dest = Path::new(destination).join(filename);
        let mut file = File::create(&dest).expect(&format!("Failed to create file: {:?}", dest));
        file.write_all(fragment_header.field_data).unwrap();
    }
}

fn create(source: &str, wld_filename: &str, format: &Format) {
    let mut reader = File::open(source).expect(&format!("Could not open source file: {}", source));
    let wld_doc: WldDoc = match format {
        Format::Raw => {
            let mut buff = vec![];
            reader
                .read_to_end(&mut buff)
                .expect("Could not read source file");
            parser::WldDoc::parse(&buff)
                .map_err(|e| {
                    for error in e.iter() {
                        print_error(error).unwrap();
                    }
                })
                .expect("Could not read wld file");
            todo!("Implement create from raw")
        }
        Format::Json => serde_json::from_reader(reader).expect("Could not deserialize from json"),
        Format::Ron => ron::de::from_reader(reader).expect("Could not deserialize from ron"),
    };
    let mut out = File::create(wld_filename).expect("Could not create wld file");
    out.write_all(&wld_doc.into_bytes())
        .expect("Failed to write to wld file");
}

fn stats(wld_filename: &str) -> Result<(), Box<dyn Error>> {
    let file = read_wld_file(wld_filename)?;
    let fragment_headers = parser::WldDoc::fragment_headers_by_offset(&file);
    println!("Index, Offset, Type, Size");
    for (idx, (k, v)) in fragment_headers.iter().enumerate() {
        println!(
            "{}, {:#010x}, {:#04x}, {:#010x}",
            idx, k, v.fragment_type, v.size
        );
    }
    Ok(())
}

fn read_wld_file(filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut file = File::open(filename)?;
    let mut wld_data = Vec::new();
    file.read_to_end(&mut wld_data)?;
    Ok(wld_data)
}
