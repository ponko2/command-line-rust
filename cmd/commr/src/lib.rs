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

    let mut lines1 = open(file1)?.lines().map_while(Result::ok);
    let mut lines2 = open(file2)?.lines().map_while(Result::ok);

    let mut print = |col: Column| -> Result<()> {
        let mut columns = vec![];
        match col {
            Col1(val) => {
                if options.show_col1 {
                    columns.push(val);
                }
            }
            Col2(val) => {
                if options.show_col2 {
                    if options.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
            Col3(val) => {
                if options.show_col3 {
                    if options.show_col1 {
                        columns.push("");
                    }
                    if options.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        };

        if !columns.is_empty() {
            writeln!(writer, "{}", columns.join(&options.delimiter))?;
        }

        Ok(())
    };

    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    let cmp = |a: &str, b: &str| {
        if options.insensitive {
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            a.cmp(b)
        }
    };

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(val1), Some(val2)) => match cmp(val1, val2) {
                Equal => {
                    print(Col3(val1))?;
                    line1 = lines1.next();
                    line2 = lines2.next();
                }
                Less => {
                    print(Col1(val1))?;
                    line1 = lines1.next();
                }
                Greater => {
                    print(Col2(val2))?;
                    line2 = lines2.next();
                }
            },
            (Some(val1), None) => {
                print(Col1(val1))?;
                line1 = lines1.next();
            }
            (None, Some(val2)) => {
                print(Col2(val2))?;
                line2 = lines2.next();
            }
            _ => (),
        }
    }

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin().lock()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|err| anyhow!("{filename}: {err}"))?,
        ))),
    }
}
