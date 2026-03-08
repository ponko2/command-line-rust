use anyhow::Result;
use clap::Parser;
use grepr::Options;

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

fn main() {
    if let Err(err) = run(Args::parse()) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let options = args.into();
    grepr::run(&options)
}
