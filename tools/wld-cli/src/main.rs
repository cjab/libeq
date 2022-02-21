#![feature(iter_intersperse)]

mod app;
mod event;
mod handlers;
mod ui;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use clap::{App as Clapp, Arg};

use crate::{app::App, event::Events};
use std::{error::Error, fmt, io};
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, style::Color, Terminal};

use eq_wld::parser;

#[derive(Debug)]
enum FragmentType {
    TextureImages,
    Texture,
    TextureReference,
    TwoDimensionalObject,
    TwoDimensionalObjectReference,
    Camera,
    CameraReference,
    SkeletonTrackSet,
    SkeletonTrackSetReference,
    MobSkeletonPieceTrack,
    MobSkeletonPieceTrackReference,
    ModelReferencePlayerInfo,
    ObjectLocation,
    ZoneUnknown,
    PolygonAnimation,
    PolygonAnimationReference,
    LightSource,
    LightSourceReference,
    BspTree,
    BspRegion,
    LightInfo,
    RegionFlag,
    AmbientLight,
    AlternateMesh,
    MeshReference,
    MeshAnimatedVerticesReference,
    Material,
    MaterialList,
    VertexColor,
    VertexColorReference,
    FirstFragment,
    Mesh,
    MeshAnimatedVertices,
    Unknown,
}

impl fmt::Display for FragmentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FragmentType {
    fn new(id: u32) -> FragmentType {
        match id {
            0x03 => FragmentType::TextureImages,
            0x04 => FragmentType::Texture,
            0x05 => FragmentType::TextureReference,
            0x06 => FragmentType::TwoDimensionalObject,
            0x07 => FragmentType::TwoDimensionalObjectReference,
            0x08 => FragmentType::Camera,
            0x09 => FragmentType::CameraReference,
            0x10 => FragmentType::SkeletonTrackSet,
            0x11 => FragmentType::SkeletonTrackSetReference,
            0x12 => FragmentType::MobSkeletonPieceTrack,
            0x13 => FragmentType::MobSkeletonPieceTrackReference,
            0x14 => FragmentType::ModelReferencePlayerInfo,
            0x15 => FragmentType::ObjectLocation,
            0x16 => FragmentType::ZoneUnknown,
            0x17 => FragmentType::PolygonAnimation,
            0x18 => FragmentType::PolygonAnimationReference,
            0x1b => FragmentType::LightSource,
            0x1c => FragmentType::LightSourceReference,
            0x21 => FragmentType::BspTree,
            0x22 => FragmentType::BspRegion,
            0x28 => FragmentType::LightInfo,
            0x29 => FragmentType::RegionFlag,
            0x2a => FragmentType::AmbientLight,
            0x2c => FragmentType::AlternateMesh,
            0x2d => FragmentType::MeshReference,
            0x2f => FragmentType::MeshAnimatedVerticesReference,
            0x30 => FragmentType::Material,
            0x31 => FragmentType::MaterialList,
            0x32 => FragmentType::VertexColor,
            0x33 => FragmentType::VertexColorReference,
            0x35 => FragmentType::FirstFragment,
            0x36 => FragmentType::Mesh,
            0x37 => FragmentType::MeshAnimatedVertices,
            _ => FragmentType::Unknown,
        }
    }

    fn color(&self) -> Color {
        match self {
            FragmentType::TextureImages => Color::Rgb(0xa9, 0xa9, 0xa9),
            FragmentType::Texture => Color::Rgb(0x2f, 0x4f, 0x4f),
            FragmentType::TextureReference => Color::Rgb(0x8b, 0x45, 0x13),
            FragmentType::TwoDimensionalObject => Color::Rgb(0x00, 0x64, 0x00),
            FragmentType::TwoDimensionalObjectReference => Color::Rgb(0x80, 0x80, 0x00),
            FragmentType::Camera => Color::Rgb(0x48, 0x3d, 0x8b),
            FragmentType::CameraReference => Color::Rgb(0xb2, 0x22, 0x22),
            FragmentType::SkeletonTrackSet => Color::Rgb(0x3c, 0xb3, 0x71),
            FragmentType::SkeletonTrackSetReference => Color::Rgb(0x00, 0x8b, 0x8b),
            FragmentType::MobSkeletonPieceTrack => Color::Rgb(0x00, 0x00, 0x8b),
            FragmentType::MobSkeletonPieceTrackReference => Color::Rgb(0x32, 0xcd, 0x32),
            FragmentType::ModelReferencePlayerInfo => Color::Rgb(0xda, 0xa5, 0x20),
            FragmentType::ObjectLocation => Color::Rgb(0x8b, 0x00, 0x8b),
            FragmentType::ZoneUnknown => Color::Rgb(0xb0, 0x30, 0x60),
            FragmentType::PolygonAnimation => Color::Rgb(0xff, 0x45, 0x00),
            FragmentType::PolygonAnimationReference => Color::Rgb(0xff, 0x8c, 0x00),
            FragmentType::LightSource => Color::Rgb(0xff, 0xff, 0x00),
            FragmentType::LightSourceReference => Color::Rgb(0x00, 0xff, 0x00),
            FragmentType::BspTree => Color::Rgb(0x00, 0xfa, 0x9a),
            FragmentType::BspRegion => Color::Rgb(0x00, 0xff, 0xff),
            FragmentType::LightInfo => Color::Rgb(0x00, 0xbf, 0xff),
            FragmentType::RegionFlag => Color::Rgb(0x00, 0x00, 0xff),
            FragmentType::AmbientLight => Color::Rgb(0xa0, 0x20, 0xf0),
            FragmentType::AlternateMesh => Color::Rgb(0xad, 0xff, 0x2f),
            FragmentType::MeshReference => Color::Rgb(0xff, 0x7f, 0x50),
            FragmentType::MeshAnimatedVerticesReference => Color::Rgb(0xff, 0x00, 0xff),
            FragmentType::Material => Color::Rgb(0xf0, 0xe6, 0x8c),
            FragmentType::MaterialList => Color::Rgb(0x64, 0x95, 0xed),
            FragmentType::VertexColor => Color::Rgb(0xdd, 0xa0, 0xdd),
            FragmentType::VertexColorReference => Color::Rgb(0xff, 0x14, 0x93),
            FragmentType::FirstFragment => Color::Rgb(0x7b, 0x68, 0xee),
            FragmentType::Mesh => Color::Rgb(0xaf, 0xee, 0xee),
            FragmentType::MeshAnimatedVertices => Color::Rgb(0xff, 0xe4, 0xc4),
            _ => Color::Rgb(0xff, 0xb6, 0xc1),
        }
    }
}

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

    if show_stats {
        let stats = wld_doc
            .fragments
            .iter()
            .fold(HashMap::new(), |mut map, header| {
                map.entry(header.fragment_type)
                    .or_insert_with(|| Vec::new())
                    .push(header);
                map
            });
        let mut sorted_keys: Vec<_> = stats.keys().collect();
        sorted_keys.sort();
        for k in sorted_keys {
            println!("0x{:02x?}: {}", k, stats[k].len());
        }
        return Ok(());
    }

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
