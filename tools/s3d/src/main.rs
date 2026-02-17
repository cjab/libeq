use std::fs::File;
use std::process;

use lexopt::prelude::*;
use libeq_archive::EqArchiveReader;

mod list;
mod verify;

enum Command {
    List {
        files: Vec<String>,
        verbosity: u8,
        human: bool,
    },
    Verify {
        files: Vec<String>,
        verbose: bool,
        round_trip: bool,
    },
}

pub(crate) fn open_archive(path: &str) -> Option<(EqArchiveReader<File>, Vec<String>)> {
    let file = File::open(path)
        .map_err(|e| eprintln!("{}: {}", path, e))
        .ok()?;
    let mut reader = EqArchiveReader::open(file)
        .map_err(|e| eprintln!("{}: {}", path, e))
        .ok()?;
    let filenames = reader
        .filenames()
        .map_err(|e| eprintln!("{}: {}", path, e))
        .ok()?;
    Some((reader, filenames))
}

fn parse_args() -> Result<Command, lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();

    let subcommand = match parser.next()? {
        Some(Value(val)) => val.string()?,
        Some(other) => return Err(other.unexpected()),
        None => return Err("expected subcommand: list, verify".into()),
    };

    match subcommand.as_str() {
        "list" => {
            let mut files = Vec::new();
            let mut verbosity: u8 = 0;
            let mut human = false;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('v') | Long("verbose") => {
                        verbosity = verbosity.saturating_add(1).min(2);
                    }
                    Short('h') | Long("human-readable") => {
                        human = true;
                    }
                    Value(val) => files.push(val.string()?),
                    other => return Err(other.unexpected()),
                }
            }
            if files.is_empty() {
                return Err("list requires at least one file".into());
            }
            Ok(Command::List {
                files,
                verbosity,
                human,
            })
        }
        "verify" => {
            let mut files = Vec::new();
            let mut verbose = false;
            let mut round_trip = false;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('v') | Long("verbose") => {
                        verbose = true;
                    }
                    Short('r') | Long("round-trip") => {
                        round_trip = true;
                    }
                    Value(val) => files.push(val.string()?),
                    other => return Err(other.unexpected()),
                }
            }
            if files.is_empty() {
                return Err("verify requires at least one file".into());
            }
            Ok(Command::Verify {
                files,
                verbose,
                round_trip,
            })
        }
        _ => Err(format!("unknown subcommand: {}", subcommand).into()),
    }
}

fn main() {
    let command = match parse_args() {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    match command {
        Command::List {
            ref files,
            verbosity,
            human,
        } => list::run(files, verbosity, human),
        Command::Verify {
            ref files,
            verbose,
            round_trip,
        } => {
            if !verify::run(files, verbose, round_trip) {
                process::exit(1);
            }
        }
    }
}
