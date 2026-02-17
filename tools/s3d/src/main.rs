use std::process;

use lexopt::prelude::*;

mod list;

enum Command {
    List {
        files: Vec<String>,
        verbosity: u8,
        human: bool,
    },
}

fn parse_args() -> Result<Command, lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();

    let subcommand = match parser.next()? {
        Some(Value(val)) => val.string()?,
        Some(other) => return Err(other.unexpected()),
        None => return Err("expected subcommand: list".into()),
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
    }
}
