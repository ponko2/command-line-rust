use anyhow::Result;
use clap::Parser;
use echor::Options;

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

fn main() {
    if let Err(err) = run(Args::parse()) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let options = args.into();
    echor::run(&options)
}
