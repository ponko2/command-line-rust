use anyhow::Result;
use clap::Parser;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
};
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
    let mut writer: Box<dyn Write> = match args.out_file.as_deref() {
        Some(out_name) => Box::new(BufWriter::new(File::create(out_name)?)),
        _ => {
            let stdout = io::stdout();
            Box::new(BufWriter::new(stdout.lock()))
        }
    };
    let options = args.into();
    uniqr::run(&mut writer, &options)?;
    writer.flush()?;
    Ok(())
}
