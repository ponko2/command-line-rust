use anyhow::Result;
use clap::{ArgAction, Parser};
use commr::Options;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `comm`
struct Args {
    /// Input file 1
    #[arg()]
    file1: String,

    /// Input file 2
    #[arg()]
    file2: String,

    /// Suppress printing of column 1
    #[arg(short = '1', action = ArgAction::SetFalse)]
    show_col1: bool,

    /// Suppress printing of column 2
    #[arg(short = '2', action = ArgAction::SetFalse)]
    show_col2: bool,

    /// Suppress printing of column 3
    #[arg(short = '3', action = ArgAction::SetFalse)]
    show_col3: bool,

    /// Case-insensitive comparison of lines
    #[arg(short)]
    insensitive: bool,

    /// Output delimiter
    #[arg(short, long = "output-delimiter", default_value = "\t")]
    delimiter: String,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            file1: args.file1,
            file2: args.file2,
            show_col1: args.show_col1,
            show_col2: args.show_col2,
            show_col3: args.show_col3,
            insensitive: args.insensitive,
            delimiter: args.delimiter,
        }
    }
}

fn main() {
    if let Err(err) = run(Args::parse()) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let options = args.into();
    commr::run(&options)
}
