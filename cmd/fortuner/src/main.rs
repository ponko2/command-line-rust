use anyhow::Result;
use clap::Parser;
use fortuner::Options;
use std::io;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `fortune`
struct Args {
    /// Input files or directories
    #[arg(required = true, value_name = "FILE")]
    sources: Vec<String>,

    /// Pattern
    #[arg(short = 'm', long)]
    pattern: Option<String>,

    /// Case-insensitive pattern matching
    #[arg(short, long)]
    insensitive: bool,

    /// Random seed
    #[arg(short, long, value_parser = clap::value_parser!(u64))]
    seed: Option<u64>,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            sources: args.sources,
            pattern: args.pattern,
            insensitive: args.insensitive,
            seed: args.seed,
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
    let options = args.into();
    fortuner::run(&options)
}
