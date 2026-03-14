use anyhow::Result;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
};

#[derive(Debug)]
pub struct Options {
    pub files: Vec<String>,
    pub lines: u64,
    pub bytes: Option<u64>,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let num_files = options.files.len();

    for (file_num, filename) in options.files.iter().enumerate() {
        let Ok(mut file) = open(filename).inspect_err(|err| eprintln!("{filename}: {err}")) else {
            continue;
        };

        if num_files > 1 {
            writeln!(
                writer,
                "{}==> {filename} <==",
                if file_num > 0 { "\n" } else { "" },
            )?;
        }

        if let Some(num_bytes) = options.bytes {
            let mut buffer = vec![0; num_bytes as usize];
            let bytes_read = file.read(&mut buffer)?;
            write!(writer, "{}", String::from_utf8_lossy(&buffer[..bytes_read]))?;
            continue;
        }

        let mut line = String::new();
        for _ in 0..options.lines {
            let bytes = file.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            write!(writer, "{line}")?;
            line.clear();
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
