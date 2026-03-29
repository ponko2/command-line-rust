use anyhow::{Result, anyhow, bail};
use itertools::Itertools;
use rand::{SeedableRng, prelude::IndexedRandom, rngs::StdRng};
use regex::RegexBuilder;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Options {
    pub sources: Vec<String>,
    pub pattern: Option<String>,
    pub insensitive: bool,
    pub seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let pattern = options
        .pattern
        .as_deref()
        .map(|val| {
            RegexBuilder::new(val)
                .case_insensitive(options.insensitive)
                .build()
                .map_err(|_| anyhow!(r#"Invalid --pattern "{val}""#))
        })
        .transpose()?;

    let files = find_files(&options.sources)?;
    let fortunes = read_fortunes(&files)?;

    let Some(pattern) = pattern else {
        writeln!(
            writer,
            "{}",
            pick_fortune(&fortunes, options.seed)
                .unwrap_or_else(|| "No fortunes found".to_string())
        )?;
        return Ok(());
    };

    for (source, group) in &fortunes
        .iter()
        .filter(|fortune| pattern.is_match(&fortune.text))
        .chunk_by(|fortune| &fortune.source)
    {
        eprintln!("({})\n%", source);
        for fortune in group {
            writeln!(writer, "{}\n%", fortune.text)?;
        }
    }

    Ok(())
}

fn find_files(paths: &[String]) -> Result<Vec<PathBuf>> {
    let dat = OsStr::new("dat");
    let mut files = vec![];

    for path in paths {
        if let Err(err) = fs::metadata(path) {
            bail!("{path}: {err}");
        }

        files.extend(
            WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file() && e.path().extension() != Some(dat))
                .map(|e| e.into_path()),
        )
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn read_fortunes(paths: &[PathBuf]) -> Result<Vec<Fortune>> {
    let mut fortunes = vec![];
    let mut buffer = String::new();

    for path in paths {
        let basename = path.file_name().unwrap().to_string_lossy().into_owned();
        let file = File::open(path).map_err(|err| anyhow!("{path:?}: {err}"))?;

        let mut reader = LineReader::new(BufReader::new(file));
        while let Some(line) = reader.read_line()? {
            if line == "%" {
                if !buffer.is_empty() {
                    fortunes.push(Fortune {
                        source: basename.clone(),
                        text: buffer.clone(),
                    });
                    buffer.clear();
                }
            } else {
                if !buffer.is_empty() {
                    buffer.push('\n');
                }
                buffer.push_str(line);
            }
        }
    }

    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    let mut rng = seed
        .map(StdRng::seed_from_u64)
        .unwrap_or_else(|| StdRng::from_rng(&mut rand::rng()));

    fortunes.choose(&mut rng).map(|f| f.text.clone())
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

#[cfg(test)]
mod tests {
    use super::{Fortune, find_files, pick_fortune, read_fortunes};

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.first().unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.first().unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // Parses all the fortunes without a filter
        let res = read_fortunes(&["./tests/inputs/jokes".into()]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\nA. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\nA: A bad idea (bad-eye deer)."
            );
        }

        // Filters for matching text
        let res = read_fortunes(&[
            "./tests/inputs/jokes".into(),
            "./tests/inputs/quotes".into(),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
