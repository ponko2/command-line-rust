use anyhow::{Result, anyhow};
use regex::{Regex, RegexBuilder};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    mem,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Options {
    pub pattern: String,
    pub files: Vec<String>,
    pub insensitive: bool,
    pub recursive: bool,
    pub count: bool,
    pub invert: bool,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let pattern = RegexBuilder::new(&options.pattern)
        .case_insensitive(options.insensitive)
        .build()
        .map_err(|_| anyhow!(r#"Invalid pattern "{}""#, options.pattern))?;

    let entries = find_files(&options.files, options.recursive);
    let num_files = entries.len();
    let mut print = |fname: &str, val: &str| -> Result<()> {
        if num_files > 1 {
            write!(writer, "{fname}:{val}")?;
        } else {
            write!(writer, "{val}")?;
        }
        Ok(())
    };

    for entry in entries {
        let Ok(filename) = entry.inspect_err(|err| eprintln!("{err}")) else {
            continue;
        };

        let Ok(file) = open(&filename).inspect_err(|err| eprintln!("{err}")) else {
            continue;
        };

        let Ok(matches) =
            find_lines(file, &pattern, options.invert).inspect_err(|err| eprintln!("{err}"))
        else {
            continue;
        };

        if options.count {
            print(&filename, &format!("{}\n", matches.len()))?;
            continue;
        }

        for line in &matches {
            print(&filename, line)?;
        }
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

fn find_lines<T: BufRead>(mut file: T, pattern: &Regex, invert: bool) -> Result<Vec<String>> {
    let mut matches = vec![];
    let mut line = String::new();

    loop {
        let bytes = file.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if pattern.is_match(&line) ^ invert {
            matches.push(mem::take(&mut line));
        }
        line.clear();
    }

    Ok(matches)
}

fn find_files(paths: &[String], recursive: bool) -> Vec<Result<String>> {
    let mut results = vec![];

    for path in paths {
        if path == "-" {
            results.push(Ok(path.clone()));
            continue;
        }

        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(err) => {
                results.push(Err(anyhow!("{path}: {err}")));
                continue;
            }
        };

        if metadata.is_file() {
            results.push(Ok(path.to_string()));
            continue;
        }

        if !metadata.is_dir() {
            continue;
        }

        if !recursive {
            results.push(Err(anyhow!("{path} is a directory")));
            continue;
        }

        results.extend(
            WalkDir::new(path)
                .into_iter()
                .flatten()
                .filter(|e| e.file_type().is_file())
                .map(|e| Ok(e.path().display().to_string())),
        );
    }

    results
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use pretty_assertions::assert_eq;
    use rand::{RngExt, distr::Alphanumeric};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(err) = &files[0] {
            assert_eq!(err.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
