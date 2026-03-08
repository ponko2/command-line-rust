use anyhow::Result;
use catr::Options;
use clap::Parser;
use std::io;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `cat`
struct Args {
    /// Input file(s)
    #[arg(value_name = "FILE", default_value = "-")]
    files: Vec<String>,

    /// Number lines
    #[arg(short = 'n', long = "number", conflicts_with = "number_nonblank_lines")]
    number_lines: bool,

    /// Number non-blank lines
    #[arg(short = 'b', long = "number-nonblank")]
    number_nonblank_lines: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            files: args.files,
            number_lines: args.number_lines,
            number_nonblank_lines: args.number_nonblank_lines,
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
    catr::run(&options)
}
