use std::io::Write;

use anyhow::Result;
use clap::ValueEnum;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug)]
pub struct Options {
    pub paths: Vec<String>,
    pub names: Vec<Regex>,
    pub entry_types: Vec<EntryType>,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum EntryType {
    #[clap(name = "d")]
    Dir,

    #[clap(name = "f")]
    File,

    #[clap(name = "l")]
    Link,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let type_filter = |entry: &DirEntry| {
        options.entry_types.is_empty()
            || options
                .entry_types
                .iter()
                .any(|entry_type| match entry_type {
                    EntryType::Link => entry.file_type().is_symlink(),
                    EntryType::Dir => entry.file_type().is_dir(),
                    EntryType::File => entry.file_type().is_file(),
                })
    };

    let name_filter = |entry: &DirEntry| {
        options.names.is_empty()
            || options
                .names
                .iter()
                .any(|re| re.is_match(&entry.file_name().to_string_lossy()))
    };

    for path in &options.paths {
        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|res| res.inspect_err(|err| eprintln!("{err}")).ok())
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        writeln!(writer, "{}", entries.join("\n"))?;
    }

    Ok(())
}
