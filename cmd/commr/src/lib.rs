use crate::Column::*;
use anyhow::{Result, anyhow, bail};
use std::{
    cmp::Ordering::*,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

#[derive(Debug)]
pub struct Options {
    pub file1: String,
    pub file2: String,
    pub show_col1: bool,
    pub show_col2: bool,
    pub show_col3: bool,
    pub insensitive: bool,
    pub delimiter: String,
}

enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let file1 = &options.file1;
    let file2 = &options.file2;

    if file1 == "-" && file2 == "-" {
        bail!(r#"Both input files cannot be STDIN ("-")"#);
    }

    let mut print = |col: Column| -> Result<()> {
        let (val, show, left_columns) = match col {
            Col1(val) => (val, options.show_col1, 0),
            Col2(val) => (val, options.show_col2, options.show_col1 as u8),
            Col3(val) => (
                val,
                options.show_col3,
                options.show_col1 as u8 + options.show_col2 as u8,
            ),
        };

        if show {
            for _ in 0..left_columns {
                write!(writer, "{}", options.delimiter)?;
            }
            writeln!(writer, "{val}")?;
        }

        Ok(())
    };

    let cmp = |a: &str, b: &str| {
        if !options.insensitive {
            return a.cmp(b);
        }
        let a = a.chars().flat_map(char::to_lowercase);
        let b = b.chars().flat_map(char::to_lowercase);
        a.cmp(b)
    };

    let mut reader1 = LineReader::new(open(file1)?);
    let mut reader2 = LineReader::new(open(file2)?);
    let mut line1 = reader1.read_line()?;
    let mut line2 = reader2.read_line()?;

    while line1.is_some() || line2.is_some() {
        match (line1, line2) {
            (Some(val1), Some(val2)) => match cmp(val1, val2) {
                Equal => {
                    print(Col3(val1))?;
                    line1 = reader1.read_line()?;
                    line2 = reader2.read_line()?;
                }
                Less => {
                    print(Col1(val1))?;
                    line1 = reader1.read_line()?;
                }
                Greater => {
                    print(Col2(val2))?;
                    line2 = reader2.read_line()?;
                }
            },
            (Some(val1), None) => {
                print(Col1(val1))?;
                line1 = reader1.read_line()?;
            }
            (None, Some(val2)) => {
                print(Col2(val2))?;
                line2 = reader2.read_line()?;
            }
            _ => {}
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
