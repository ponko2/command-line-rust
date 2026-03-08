use anyhow::Result;
use clap::Parser;
use lsr::Options;
use std::io;

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
    lsr::run(&options)
}
