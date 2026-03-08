use anyhow::Result;
use clap::Parser;
use std::io::{self, BufWriter, Write};
use wcr::Options;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `wc`
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Show line count
    #[arg(short, long)]
    lines: bool,

    /// Show word count
    #[arg(short, long)]
    words: bool,

    /// Show byte count
    #[arg(short = 'c', long)]
    bytes: bool,

    /// Show character count
    #[arg(short = 'm', long, conflicts_with = "bytes")]
    chars: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            files: args.files,
            lines: args.lines,
            words: args.words,
            bytes: args.bytes,
            chars: args.chars,
        }
    }
}

fn main() {
    let Err(err) = run(Args::parse()) else {
        return;
    };

    // Handle broken pipe gracefully
    if err
        .downcast_ref::<io::Error>()
        .is_some_and(|err| err.kind() == io::ErrorKind::BrokenPipe)
    {
        return;
    }

    eprintln!("{err}");
    std::process::exit(1);
}

fn run(args: Args) -> Result<()> {
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let options = args.into();
    wcr::run(&mut writer, &options)?;
    writer.flush()?;
    Ok(())
}
