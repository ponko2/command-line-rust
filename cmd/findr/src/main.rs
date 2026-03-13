use anyhow::Result;
use clap::{ArgAction, Parser};
use findr::{EntryType, Options};
use regex::Regex;
use std::io::{self, BufWriter};

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `find`
struct Args {
    /// Search path(s)
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    /// Names
    #[arg(
        short = 'n',
        long = "name",
        value_name = "NAME",
        value_parser = Regex::new,
        action = ArgAction::Append,
        num_args = 0..,
    )]
    names: Vec<Regex>,

    /// Entry types
    #[arg(
        short = 't',
        long = "type",
        value_name = "TYPE",
        value_parser = clap::value_parser!(EntryType),
        action = ArgAction::Append,
        num_args = 0..,
    )]
    entry_types: Vec<EntryType>,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            paths: args.paths,
            names: args.names,
            entry_types: args.entry_types,
        }
    }
}

use std::process::ExitCode;

fn main() -> ExitCode {
    let Err(err) = run(Args::parse()) else {
        return ExitCode::SUCCESS;
    };

    // Handle broken pipe gracefully
    if err
        .downcast_ref::<io::Error>()
        .is_some_and(|err| err.kind() == io::ErrorKind::BrokenPipe)
    {
        return ExitCode::SUCCESS;
    }

    eprintln!("{err}");
    ExitCode::FAILURE
}

fn run(args: Args) -> Result<()> {
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let options = args.into();
    findr::run(&mut writer, &options)
}
