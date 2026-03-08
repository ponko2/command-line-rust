use anyhow::Result;
use clap::Parser;
use std::io;
use uniqr::Options;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `uniq`
struct Args {
    /// Input file
    #[arg(value_name = "IN_FILE", default_value = "-")]
    in_file: String,

    /// Output file
    #[arg(value_name = "OUT_FILE")]
    out_file: Option<String>,

    /// Show counts
    #[arg(short, long)]
    count: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            in_file: args.in_file,
            out_file: args.out_file,
            count: args.count,
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
    uniqr::run(&options)
}
