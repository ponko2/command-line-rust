use anyhow::Result;
use clap::Parser;
use cutr::{Options, OptionsExtract};
use std::io::{self, BufWriter, Write};

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rust version of `cut`
struct Args {
    /// Input file(s)
    #[arg(default_value = "-")]
    files: Vec<String>,

    /// Field delimiter
    #[arg(short, long, value_name = "DELIMITER", default_value = "\t")]
    delimiter: String,

    #[command(flatten)]
    extract: ArgsExtract,
}

impl From<Args> for Options {
    fn from(args: Args) -> Self {
        Self {
            files: args.files,
            delimiter: args.delimiter,
            extract: args.extract.into(),
        }
    }
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct ArgsExtract {
    /// Selected fields
    #[arg(short, long, value_name = "FIELDS")]
    fields: Option<String>,

    /// Selected bytes
    #[arg(short, long, value_name = "BYTES")]
    bytes: Option<String>,

    /// Selected chars
    #[arg(short, long, value_name = "CHARS")]
    chars: Option<String>,
}

impl From<ArgsExtract> for OptionsExtract {
    fn from(args: ArgsExtract) -> Self {
        Self {
            fields: args.fields,
            bytes: args.bytes,
            chars: args.chars,
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
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    let options = args.into();
    cutr::run(&mut writer, &options)?;
    writer.flush()?;
    Ok(())
}
