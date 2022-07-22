use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};

use clap::{ArgGroup, CommandFactory, ErrorKind, Parser};

use libeq_archive::EqArchive;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("action")
        .required(true)
        .args(&["extract", "create"])))]
struct Cli {
    /// Extract archive
    #[clap(short = 'x', long, action)]
    extract: bool,

    /// Create a new archive
    #[clap(short, long, action)]
    create: bool,

    /// Source archive when extracting or directory when creating
    #[clap(value_parser)]
    source: PathBuf,

    /// Destination directory when extracting or archive when creating
    #[clap(value_parser)]
    destination: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();

    let result = if cli.extract {
        extract(cli.source, cli.destination)
    } else if cli.create {
        create(cli.source, cli.destination)
    } else {
        Ok(())
    };

    if let Err(err) = result {
        let message = match err {
            CliError::InvalidArgument(msg) => msg,
            CliError::Destination(io_error) => format!("DESTINATION: {}", io_error),
            CliError::Source(io_error) => format!("SOURCE: {}", io_error),
            CliError::Archive(archive_error) => format!("{:?}", archive_error),
        };
        let mut cmd = Cli::command();
        cmd.error(ErrorKind::InvalidValue, message).exit();
    };

    Ok(())
}

enum CliError {
    InvalidArgument(String),
    Destination(io::Error),
    Source(io::Error),
    Archive(libeq_archive::Error),
}

fn extract(source: PathBuf, destination: PathBuf) -> Result<(), CliError> {
    if source.is_dir() || source.extension() != Some(OsStr::new("s3d")) {
        return Err(CliError::InvalidArgument(
            "SOURCE must be an .s3d file when using --extract".into(),
        ));
    }

    fs::create_dir_all(&destination).map_err(|err| CliError::Destination(err))?;
    let archive_file = fs::File::open(&source).map_err(|err| CliError::Source(err))?;
    let archive = EqArchive::read(archive_file).map_err(|err| CliError::Archive(err))?;
    let destination_path = Path::new(&destination);
    for (filename, data) in archive.iter() {
        let path = destination_path.join(filename);
        let mut file = fs::File::create(&path).map_err(|err| CliError::Destination(err))?;
        file.write_all(&data)
            .map_err(|err| CliError::Destination(err))?;
    }

    Ok(())
}

fn create(source: PathBuf, destination: PathBuf) -> Result<(), CliError> {
    if !source.is_dir() {
        return Err(CliError::InvalidArgument(
            "SOURCE must be a directory when using --create".into(),
        ));
    }

    let source_dir = fs::read_dir(&source).map_err(|err| CliError::Source(err))?;
    let mut archive_file =
        fs::File::create(&destination).map_err(|err| CliError::Destination(err))?;
    let mut archive = EqArchive::new();
    for entry in source_dir {
        let entry = entry.map_err(|err| CliError::Source(err))?;
        let path = entry.path();
        let filename = entry.file_name().to_str().unwrap().to_string();
        let mut file = fs::File::open(&path).map_err(|err| CliError::Source(err))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|err| CliError::Source(err))?;
        archive.push(&filename, &data);
    }
    let bytes = archive.to_bytes().map_err(|err| CliError::Archive(err))?;
    archive_file
        .write_all(&bytes)
        .map_err(|err| CliError::Destination(err))?;

    Ok(())
}
