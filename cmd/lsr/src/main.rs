use anyhow::Result;
use clap::Parser;
use lsr::Options;
use std::io::{self, BufWriter, Write};

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `ls`
struct Args {
    /// Files and/or directories
    #[arg(default_value = ".")]
    paths: Vec<String>,

    /// Long listing
    #[arg(short, long)]
    long: bool,

    /// Show all files
    #[arg(short = 'a', long = "all")]
    show_hidden: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            paths: args.paths,
            long: args.long,
            show_hidden: args.show_hidden,
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
    lsr::run(&mut writer, &options)?;
    writer.flush()?;
    Ok(())
}
