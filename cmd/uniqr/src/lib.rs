use anyhow::{Result, anyhow};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

#[derive(Debug)]
pub struct Options {
    pub in_file: String,
    pub count: bool,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let mut file = open(&options.in_file).map_err(|err| anyhow!("{}: {err}", options.in_file))?;

    let mut print = |num: u64, text: &str| -> Result<()> {
        if num > 0 {
            if options.count {
                write!(writer, "{num:>4} {text}")?;
            } else {
                write!(writer, "{text}")?;
            }
        };
        Ok(())
    };

    let mut line = String::new();
    let mut previous = String::new();
    let mut count: u64 = 0;
    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        if line.trim_end() != previous.trim_end() {
            print(count, &previous)?;
            std::mem::swap(&mut previous, &mut line);
            count = 0;
        }

        count += 1;
        line.clear();
    }
    print(count, &previous)?;

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    if filename == "-" {
        return Ok(Box::new(BufReader::new(io::stdin().lock())));
    }
    Ok(Box::new(BufReader::new(File::open(filename)?)))
}
