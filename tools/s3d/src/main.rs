use std::error::Error;
use std::fs::File;
use std::process;

use lexopt::prelude::*;
use libeq_pfs::PfsReader;
mod create;
mod extract;
mod fmt;
mod get;
mod info;
mod list;
mod verify;

enum Command {
    List {
        files: Vec<String>,
        verbosity: u8,
        raw: bool,
    },
    Verify {
        files: Vec<String>,
        verbose: bool,
    },
    Extract {
        archive: String,
        files: Vec<String>,
        output: Option<String>,
        verbose: bool,
    },
    Create {
        archive: String,
        inputs: Vec<String>,
        verbose: bool,
        force: bool,
    },
    Get {
        archive: String,
        filename: String,
    },
    Info {
        files: Vec<String>,
        raw: bool,
    },
}

const HELP: &str = "\
s3d — EverQuest archive tool

Usage: s3d <command> [options] [args]

Commands:
  list    (ls)   List archive contents
  verify  (v)    Verify archive integrity
  extract (x)    Extract files from archive
  create  (c)    Create archive from files
  get            Extract single file to stdout
  info    (i)    Display archive metadata

Run 's3d <command> --help' for more information.";

fn print_help() {
    println!("{}", HELP);
}

fn eprint_help() {
    eprintln!("{}", HELP);
}

pub(crate) fn open_archive(path: &str) -> Result<(PfsReader<File>, Vec<String>), Box<dyn Error>> {
    let file = File::open(path).map_err(|e| format!("{}: {}", path, e))?;
    let mut reader = PfsReader::open(file).map_err(|e| format!("{}: {}", path, e))?;
    let filenames = reader.filenames().map_err(|e| format!("{}: {}", path, e))?;
    Ok((reader, filenames))
}

fn parse_args() -> Result<Command, lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();

    let subcommand = match parser.next()? {
        Some(Value(val)) => val.string()?,
        Some(Short('h') | Long("help")) => {
            print_help();
            process::exit(0);
        }
        Some(other) => return Err(other.unexpected()),
        None => {
            eprint_help();
            process::exit(1);
        }
    };

    match subcommand.as_str() {
        "list" | "ls" => {
            let mut files = Vec::new();
            let mut verbosity: u8 = 0;
            let mut raw = false;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('v') | Long("verbose") => {
                        verbosity = verbosity.saturating_add(1).min(2);
                    }
                    Short('r') | Long("raw") => {
                        raw = true;
                    }
                    Short('h') | Long("help") => {
                        list::print_help();
                        process::exit(0);
                    }
                    Value(val) => files.push(val.string()?),
                    other => return Err(other.unexpected()),
                }
            }
            if files.is_empty() {
                list::eprint_help();
                process::exit(1);
            }
            Ok(Command::List {
                files,
                verbosity,
                raw,
            })
        }
        "verify" | "v" => {
            let mut files = Vec::new();
            let mut verbose = false;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('v') | Long("verbose") => {
                        verbose = true;
                    }
                    Short('h') | Long("help") => {
                        verify::print_help();
                        process::exit(0);
                    }
                    Value(val) => files.push(val.string()?),
                    other => return Err(other.unexpected()),
                }
            }
            if files.is_empty() {
                verify::eprint_help();
                process::exit(1);
            }
            Ok(Command::Verify { files, verbose })
        }
        "extract" | "x" => {
            let mut archive = None;
            let mut files = Vec::new();
            let mut output = None;
            let mut verbose = false;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('v') | Long("verbose") => {
                        verbose = true;
                    }
                    Short('o') | Long("output") => {
                        output = Some(parser.value()?.string()?);
                    }
                    Short('h') | Long("help") => {
                        extract::print_help();
                        process::exit(0);
                    }
                    Value(val) => {
                        let s = val.string()?;
                        if archive.is_none() {
                            archive = Some(s);
                        } else {
                            files.push(s);
                        }
                    }
                    other => return Err(other.unexpected()),
                }
            }
            let Some(archive) = archive else {
                extract::eprint_help();
                process::exit(1);
            };
            Ok(Command::Extract {
                archive,
                files,
                output,
                verbose,
            })
        }
        "create" | "c" => {
            let mut archive = None;
            let mut inputs = Vec::new();
            let mut verbose = false;
            let mut force = false;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('v') | Long("verbose") => {
                        verbose = true;
                    }
                    Short('f') | Long("force") => {
                        force = true;
                    }
                    Short('h') | Long("help") => {
                        create::print_help();
                        process::exit(0);
                    }
                    Value(val) => {
                        let s = val.string()?;
                        if archive.is_none() {
                            archive = Some(s);
                        } else {
                            inputs.push(s);
                        }
                    }
                    other => return Err(other.unexpected()),
                }
            }
            if archive.is_none() || inputs.is_empty() {
                create::eprint_help();
                process::exit(1);
            }
            let archive = archive.unwrap();
            Ok(Command::Create {
                archive,
                inputs,
                verbose,
                force,
            })
        }
        "get" => {
            let mut archive = None;
            let mut filename = None;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('h') | Long("help") => {
                        get::print_help();
                        process::exit(0);
                    }
                    Value(val) => {
                        let s = val.string()?;
                        if archive.is_none() {
                            archive = Some(s);
                        } else if filename.is_none() {
                            filename = Some(s);
                        } else {
                            return Err("get accepts exactly one filename".into());
                        }
                    }
                    other => return Err(other.unexpected()),
                }
            }
            if archive.is_none() || filename.is_none() {
                get::eprint_help();
                process::exit(1);
            }
            let archive = archive.unwrap();
            let filename = filename.unwrap();
            Ok(Command::Get { archive, filename })
        }
        "info" | "i" => {
            let mut files = Vec::new();
            let mut raw = false;
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('r') | Long("raw") => {
                        raw = true;
                    }
                    Short('h') | Long("help") => {
                        info::print_help();
                        process::exit(0);
                    }
                    Value(val) => files.push(val.string()?),
                    other => return Err(other.unexpected()),
                }
            }
            if files.is_empty() {
                info::eprint_help();
                process::exit(1);
            }
            Ok(Command::Info { files, raw })
        }
        "help" => {
            // s3d help <subcommand>
            match parser.next()? {
                Some(Value(val)) => {
                    let sub = val.string()?;
                    match sub.as_str() {
                        "list" | "ls" => list::print_help(),
                        "verify" | "v" => verify::print_help(),
                        "extract" | "x" => extract::print_help(),
                        "create" | "c" => create::print_help(),
                        "get" => get::print_help(),
                        "info" | "i" => info::print_help(),
                        _ => {
                            eprintln!("unknown subcommand: {}", sub);
                            print_help();
                            process::exit(1);
                        }
                    }
                    process::exit(0);
                }
                _ => {
                    print_help();
                    process::exit(0);
                }
            }
        }
        _ => {
            eprintln!("unknown subcommand: {}", subcommand);
            eprintln!();
            eprint_help();
            process::exit(1);
        }
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
            raw,
        } => {
            if let Err(e) = list::run(files, verbosity, !raw) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Command::Verify { ref files, verbose } => {
            if let Err(e) = verify::run(files, verbose) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Command::Extract {
            ref archive,
            ref files,
            ref output,
            verbose,
        } => {
            if let Err(e) = extract::run(archive, files, output.as_deref(), verbose) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Command::Create {
            ref archive,
            ref inputs,
            verbose,
            force,
        } => {
            if let Err(e) = create::run(archive, inputs, verbose, force) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Command::Get {
            ref archive,
            ref filename,
        } => {
            if let Err(e) = get::run(archive, filename) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        Command::Info { ref files, raw } => {
            if let Err(e) = info::run(files, !raw) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}
