use anyhow::Result;
use clap::{ArgAction, Parser};
use findr::{EntryType, Options};
use regex::Regex;

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

fn main() {
    if let Err(err) = run(Args::parse()) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let options = args.into();
    findr::run(&options)
}
