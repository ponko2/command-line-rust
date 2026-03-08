use anyhow::Result;
use calr::Options;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `cal`
struct Args {
    /// Year (1-9999)
    #[arg(value_parser = clap::value_parser!(i32).range(1..=9999))]
    year: Option<i32>,

    /// Month name or number (1-12)
    #[arg(short)]
    month: Option<String>,

    /// Show the whole current year
    #[arg(short = 'y', long = "year", conflicts_with_all = ["month", "year"])]
    show_current_year: bool,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            year: args.year,
            month: args.month,
            show_current_year: args.show_current_year,
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
    calr::run(&options)
}
