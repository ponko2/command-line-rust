use anyhow::Result;
use clap::Parser;
use std::io::{self, BufWriter};
use tailr::Options;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `tail`
struct Args {
    /// Input file(s)
    #[arg(required = true)]
    files: Vec<String>,

    /// Number of lines
    #[arg(value_name = "LINES", short = 'n', long, default_value = "10")]
    lines: String,

    /// Number of bytes
    #[arg(value_name = "BYTES", short = 'c', long, conflicts_with = "lines")]
    bytes: Option<String>,

    /// Suppress headers
    #[arg(short, long)]
    quiet: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            files: args.files,
            lines: args.lines,
            bytes: args.bytes,
            quiet: args.quiet,
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
    tailr::run(&mut writer, &options)
}
