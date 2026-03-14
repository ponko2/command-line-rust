use anyhow::{Result, anyhow};
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
        let Ok(file) = open(filename).inspect_err(|err| eprintln!("{err}")) else {
            continue;
        };

        let mut prev_num = 0;
        let mut line_num = 0;
        let mut reader = LineReader::new(file);
        while let Some(line) = reader.read_line()? {
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
            line_num += 1;
        }
    }

    Ok(())
}

struct LineReader<R: BufRead> {
    reader: R,
    buffer: String,
}

impl<R: BufRead> LineReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::new(),
        }
    }

    fn read_line(&mut self) -> Result<Option<&str>> {
        self.buffer.clear();
        let n = self.reader.read_line(&mut self.buffer)?;
        Ok((n > 0).then(|| self.buffer.trim_end()))
    }
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    if filename == "-" {
        return Ok(Box::new(BufReader::new(io::stdin().lock())));
    }
    Ok(Box::new(BufReader::new(
        File::open(filename).map_err(|err| anyhow!("{filename}: {err}"))?,
    )))
}
