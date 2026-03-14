use anyhow::{Result, anyhow};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub files: Vec<String>,
    pub lines: bool,
    pub words: bool,
    pub bytes: bool,
    pub chars: bool,
}

#[derive(Debug, PartialEq)]
struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &options.files {
        let Ok(file) = open(filename).inspect_err(|err| eprintln!("{err}")) else {
            continue;
        };

        let Ok(info) = count(file) else {
            continue;
        };

        writeln!(
            writer,
            "{}{}{}{}{}",
            format_field(info.num_lines, options.lines),
            format_field(info.num_words, options.words),
            format_field(info.num_bytes, options.bytes),
            format_field(info.num_chars, options.chars),
            if filename == "-" {
                "".to_string()
            } else {
                format!(" {filename}")
            },
        )?;

        total_lines += info.num_lines;
        total_words += info.num_words;
        total_bytes += info.num_bytes;
        total_chars += info.num_chars;
    }

    if options.files.len() > 1 {
        writeln!(
            writer,
            "{}{}{}{} total",
            format_field(total_lines, options.lines),
            format_field(total_words, options.words),
            format_field(total_bytes, options.bytes),
            format_field(total_chars, options.chars)
        )?;
    }

    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    if filename == "-" {
        return Ok(Box::new(BufReader::new(io::stdin().lock())));
    }
    Ok(Box::new(BufReader::new(
        File::open(filename).map_err(|err| anyhow!("{filename}: {err}"))?,
    )))
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{value:>8}")
    } else {
        String::new()
    }
}

fn count(mut file: impl BufRead) -> Result<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;
    let mut line = String::new();

    loop {
        let line_bytes = file.read_line(&mut line)?;
        if line_bytes == 0 {
            break;
        }
        num_bytes += line_bytes;
        num_lines += 1;
        num_words += line.split_whitespace().count();
        num_chars += line.chars().count();
        line.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use super::{FileInfo, count, format_field};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world.\nI just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 2,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(10, true), "      10");
    }
}
