use anyhow::Result;
use clap::Parser;
use echor::Options;
use std::io::{self, BufWriter, Write};

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `echo`
struct Args {
    /// Input text
    #[arg(required = true)]
    text: Vec<String>,

    /// Do not print newline
    #[arg(short = 'n')]
    omit_newline: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            text: args.text,
            omit_newline: args.omit_newline,
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
    echor::run(&mut writer, &options)?;
    writer.flush()?;
    Ok(())
}
