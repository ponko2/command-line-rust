use anyhow::Result;
use clap::Parser;
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
    if let Err(err) = run(Args::parse()) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let options = args.into();
    uniqr::run(&options)
}
