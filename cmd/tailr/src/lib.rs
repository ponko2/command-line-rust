use crate::TakeValue::*;
use anyhow::{Result, anyhow, bail};
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
pub struct Options {
    pub files: Vec<String>,
    pub lines: String,
    pub bytes: Option<String>,
    pub quiet: bool,
}

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let lines = parse_num(&options.lines).map_err(|err| anyhow!("illegal line count -- {err}"))?;

    let bytes = options
        .bytes
        .as_deref()
        .map(parse_num)
        .transpose()
        .map_err(|err| anyhow!("illegal byte count -- {err}"))?;

    let num_files = options.files.len();
    for (file_num, filename) in options.files.iter().enumerate() {
        match File::open(filename) {
            Err(err) => eprintln!("{filename}: {err}"),
            Ok(file) => {
                if !options.quiet && num_files > 1 {
                    writeln!(
                        writer,
                        "{}==> {filename} <==",
                        if file_num > 0 { "\n" } else { "" },
                    )?;
                }

                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                let file = BufReader::new(file);
                if let Some(num_bytes) = &bytes {
                    print_bytes(writer, file, num_bytes, total_bytes)?;
                } else {
                    print_lines(writer, file, &lines, total_lines)?;
                }
            }
        }
    }

    Ok(())
}

fn parse_num(val: &str) -> Result<TakeValue> {
    let res = if val.starts_with(['+', '-']) {
        val.parse()
    } else {
        val.parse().map(i64::wrapping_neg)
    };

    match res {
        Ok(num) => {
            if num == 0 && val.starts_with('+') {
                Ok(PlusZero)
            } else {
                Ok(TakeNum(num))
            }
        }
        _ => bail!(val.to_string()),
    }
}

fn count_lines_bytes(filename: &str) -> Result<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut num_lines = 0;
    let mut num_bytes = 0;
    let mut buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read as i64;
        buf.clear();
    }
    Ok((num_lines, num_bytes))
}

fn print_bytes(
    writer: &mut impl Write,
    mut file: impl Read + Seek,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> Result<()> {
    if let Some(start) = get_start_index(num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            write!(writer, "{}", String::from_utf8_lossy(&buffer))?;
        }
    }

    Ok(())
}

fn print_lines(
    writer: &mut impl Write,
    mut file: impl BufRead,
    num_lines: &TakeValue,
    total_lines: i64,
) -> Result<()> {
    if let Some(start) = get_start_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;
            if bytes_read == 0 {
                break;
            }
            if line_num >= start {
                write!(writer, "{}", String::from_utf8_lossy(&buf))?;
            }
            line_num += 1;
            buf.clear();
        }
    }

    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        PlusZero => {
            if total > 0 {
                Some(0)
            } else {
                None
            }
        }
        TakeNum(num) => {
            if num == &0 || total == 0 || num > &total {
                None
            } else {
                let start = if num < &0 { total + num } else { num - 1 };
                Some(if start < 0 { 0 } else { start as u64 })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TakeValue::*, count_lines_bytes, get_start_index, parse_num};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 1);
        assert_eq!(bytes, 24);

        let res = count_lines_bytes("tests/inputs/twelve.txt");
        assert!(res.is_ok());
        let (lines, bytes) = res.unwrap();
        assert_eq!(lines, 12);
        assert_eq!(bytes, 63);
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
