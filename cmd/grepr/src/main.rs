use anyhow::Result;
use clap::Parser;
use grepr::Options;
use std::io::{self, BufWriter, Write};

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `grep`
struct Args {
    /// Search pattern
    #[arg()]
    pattern: String,

    /// Input file(s)
    #[arg(default_value = "-", value_name = "FILE")]
    files: Vec<String>,

    /// Case-insensitive
    #[arg(short, long)]
    insensitive: bool,

    /// Recursive search
    #[arg(short, long)]
    recursive: bool,

    /// Count occurrences
    #[arg(short, long)]
    count: bool,

    /// Invert match
    #[arg(short = 'v', long = "invert-match")]
    invert: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            pattern: args.pattern,
            files: args.files,
            insensitive: args.insensitive,
            recursive: args.recursive,
            count: args.count,
            invert: args.invert,
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
    grepr::run(&mut writer, &options)?;
    writer.flush()?;
    Ok(())
}
