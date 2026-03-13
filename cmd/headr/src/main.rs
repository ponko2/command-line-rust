use anyhow::Result;
use clap::Parser;
use headr::Options;
use std::io::{self, BufWriter};

#[derive(Parser, Debug)]
#[command(version, about)]
/// Rust version of `head`
struct Args {
    /// Input file(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,

    /// Number of lines
    #[arg(
        short('n'),
        long,
        default_value = "10",
        value_name = "LINES",
        value_parser = clap::value_parser!(u64).range(1..),
    )]
    lines: u64,

    /// Number of bytes
    #[arg(
        short('c'),
        long,
        value_name = "BYTES",
        conflicts_with = "lines",
        value_parser = clap::value_parser!(u64).range(1..),
    )]
    bytes: Option<u64>,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            files: args.files,
            lines: args.lines,
            bytes: args.bytes,
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
    headr::run(&mut writer, &options)
}
