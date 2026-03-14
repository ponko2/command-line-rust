use anyhow::Result;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

#[derive(Debug)]
pub struct Options {
    pub files: Vec<String>,
    pub number_lines: bool,
    pub number_nonblank_lines: bool,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    for filename in &options.files {
        let Ok(file) = open(filename).inspect_err(|err| eprintln!("{filename}: {err}")) else {
            continue;
        };

        let mut prev_num = 0;
        for (line_num, line_result) in file.lines().enumerate() {
            let line = line_result?;
            if options.number_lines {
                writeln!(writer, "{:6}\t{line}", line_num + 1)?;
            } else if options.number_nonblank_lines {
                if line.is_empty() {
                    writeln!(writer)?;
                } else {
                    prev_num += 1;
                    writeln!(writer, "{prev_num:6}\t{line}")?;
                }
            } else {
                writeln!(writer, "{line}")?;
            }
        }
    }

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    if filename == "-" {
        return Ok(Box::new(BufReader::new(io::stdin().lock())));
    }
    Ok(Box::new(BufReader::new(File::open(filename)?)))
}
