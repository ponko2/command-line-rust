use anyhow::{Result, anyhow, bail};
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use regex::Regex;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    num::NonZeroUsize,
    ops::Range,
    sync::LazyLock,
};

static RANGE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)-(\d+)$").unwrap());

#[derive(Debug)]
pub struct Options {
    pub files: Vec<String>,
    pub delimiter: String,
    pub extract: OptionsExtract,
}

#[derive(Debug)]
pub struct OptionsExtract {
    pub fields: Option<String>,
    pub bytes: Option<String>,
    pub chars: Option<String>,
}

type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let delim_bytes = options.delimiter.as_bytes();
    if delim_bytes.len() != 1 {
        bail!(r#"--delim "{}" must be a single byte"#, options.delimiter);
    }
    let delimiter: u8 = *delim_bytes.first().unwrap();

    let extract = if let Some(fields) = options
        .extract
        .fields
        .as_deref()
        .map(parse_pos)
        .transpose()?
    {
        Extract::Fields(fields)
    } else if let Some(bytes) = options
        .extract
        .bytes
        .as_deref()
        .map(parse_pos)
        .transpose()?
    {
        Extract::Bytes(bytes)
    } else if let Some(chars) = options
        .extract
        .chars
        .as_deref()
        .map(parse_pos)
        .transpose()?
    {
        Extract::Chars(chars)
    } else {
        unreachable!("Must have --fields, --bytes, or --chars");
    };

    for filename in &options.files {
        let Ok(file) = open(filename).inspect_err(|err| eprintln!("{err}")) else {
            continue;
        };

        match &extract {
            Extract::Fields(field_pos) => {
                let mut reader = ReaderBuilder::new()
                    .delimiter(delimiter)
                    .has_headers(false)
                    .from_reader(file);

                let mut wtr = WriterBuilder::new()
                    .delimiter(delimiter)
                    .from_writer(&mut *writer);

                for record in reader.records() {
                    wtr.write_record(extract_fields(&record?, field_pos))?;
                }
            }
            Extract::Bytes(byte_pos) => {
                let mut reader = LineReader::new(file);
                while let Some(line) = reader.read_line()? {
                    writeln!(writer, "{}", extract_bytes(line, byte_pos))?;
                }
            }
            Extract::Chars(char_pos) => {
                let mut reader = LineReader::new(file);
                while let Some(line) = reader.read_line()? {
                    writeln!(writer, "{}", extract_chars(line, char_pos))?;
                }
            }
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

// Parse an index from a string representation of an integer.
// Ensures the number is non-zero.
// Ensures the number does not start with '+'.
// Returns an index, which is a non-negative integer that is
// one less than the number represented by the original input.
fn parse_index(input: &str) -> Result<usize> {
    let value_error = || anyhow!(r#"illegal list value: "{input}""#);
    if input.starts_with('+') {
        return Err(value_error());
    }
    input
        .parse::<NonZeroUsize>()
        .map(|n| usize::from(n) - 1)
        .map_err(|_| value_error())
}

fn parse_pos(range: &str) -> Result<PositionList> {
    range
        .split(',')
        .map(|val| {
            parse_index(val).map(|n| n..n + 1).or_else(|e| {
                RANGE_RE.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        bail!(
                            "First number in range ({}) must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        );
                    }
                    Ok(n1..n2 + 1)
                })
            })
        })
        .collect::<Result<_, _>>()
}

fn extract_fields<'a>(
    record: &'a StringRecord,
    field_pos: &[Range<usize>],
) -> impl Iterator<Item = &'a str> {
    field_pos
        .iter()
        .flat_map(|range| range.clone().filter_map(|i| record.get(i)))
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();
    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| bytes.get(i)).copied())
        .collect();
    String::from_utf8_lossy(&selected).into_owned()
}

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<_> = line.chars().collect();
    char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| chars.get(i)))
        .collect()
}

#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, parse_pos};
    use csv::StringRecord;
    use pretty_assertions::assert_eq;
    use std::ops::Range;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("").is_err());

        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "+1""#,);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );

        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "1-a""#,);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a-1""#,);

        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[allow(clippy::single_range_in_vec_init)]
    #[test]
    fn test_extract_fields() {
        let rec = vec!["Captain", "Sham", "12345"].into();
        fn extract_fields<'a>(
            record: &'a StringRecord,
            field_pos: &[Range<usize>],
        ) -> Vec<&'a str> {
            super::extract_fields(record, field_pos).collect()
        }
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[allow(clippy::single_range_in_vec_init)]
    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[allow(clippy::single_range_in_vec_init)]
    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
}
