mod owner;

use anyhow::Result;
use chrono::{DateTime, Local};
use owner::Owner;
use std::{fs, io::Write, os::unix::fs::MetadataExt, path::PathBuf};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Padding, Style, object::Columns},
};
use uzers::{get_group_by_gid, get_user_by_uid};

#[derive(Debug)]
pub struct Options {
    pub paths: Vec<String>,
    pub long: bool,
    pub show_hidden: bool,
}

#[derive(Tabled)]
struct FileEntry {
    file_type: &'static str,
    #[tabled(display = "format_mode")]
    mode: u32,
    nlink: u64,
    owner: String,
    group: String,
    size: u64,
    #[tabled(display = "format_modified")]
    modified: DateTime<Local>,
    name: String,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    let paths = find_files(&options.paths, options.show_hidden)?;
    if options.long {
        writeln!(writer, "{}", format_output(&paths)?)?;
    } else {
        for path in paths {
            writeln!(writer, "{}", path.display())?;
        }
    }
    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut results = vec![];
    for name in paths {
        let Ok(meta) = fs::metadata(name).inspect_err(|err| eprintln!("{name}: {err}")) else {
            continue;
        };

        if !meta.is_dir() {
            results.push(name.into());
            continue;
        }

        for entry in fs::read_dir(name)? {
            let entry = entry?;
            let path = entry.path();
            let is_hidden = path
                .file_name()
                .is_some_and(|file_name| file_name.to_string_lossy().starts_with('.'));
            if !is_hidden || show_hidden {
                results.push(entry.path());
            }
        }
    }

    Ok(results)
}

fn format_output(paths: &[PathBuf]) -> Result<String> {
    let mut file_entries = vec![];

    for path in paths {
        let metadata = path.metadata()?;

        let uid = metadata.uid();
        let owner = get_user_by_uid(uid).map_or_else(
            || uid.to_string(),
            |u| u.name().to_string_lossy().into_owned(),
        );

        let gid = metadata.gid();
        let group = get_group_by_gid(gid).map_or_else(
            || gid.to_string(),
            |g| g.name().to_string_lossy().into_owned(),
        );

        file_entries.push(FileEntry {
            file_type: if path.is_dir() { "d" } else { "-" },
            mode: metadata.mode(),
            nlink: metadata.nlink(),
            owner,
            group,
            size: metadata.len(),
            modified: metadata.modified()?.into(),
            name: path.display().to_string(),
        });
    }

    let mut table = Table::nohead(file_entries);
    table
        .with(Style::empty())
        .with(Padding::zero())
        .modify(Columns::new(2..), Padding::new(2, 0, 0, 0))
        .modify(Columns::new(2..=2), Alignment::right())
        .modify(Columns::new(5..=5), Alignment::right());

    Ok(table.to_string())
}

/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
fn format_mode(mode: &u32) -> String {
    format!(
        "{}{}{}",
        mk_triple(*mode, Owner::User),
        mk_triple(*mode, Owner::Group),
        mk_triple(*mode, Owner::Other),
    )
}

/// Given a [`DateTime<Local>`],
/// return a string like "Mar 10 26 17:24"
fn format_modified(time: &DateTime<Local>) -> String {
    time.format("%b %d %y %H:%M").to_string()
}

/// Given an octal number like 0o500 and an [`Owner`],
/// return a string like "r-x"
fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
    format!(
        "{}{}{}",
        if mode & read == 0 { "-" } else { "r" },
        if mode & write == 0 { "-" } else { "w" },
        if mode & execute == 0 { "-" } else { "x" },
    )
}

#[cfg(test)]
mod test {
    use super::{Owner, find_files, format_mode, format_output, mk_triple};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(!parts.is_empty() && parts.len() <= 10);

        let perms = parts.first().unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";

        let res = format_output(&[bustle_path.into()]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&["tests/inputs/dir".into(), "tests/inputs/empty.txt".into()]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }

    #[test]
    fn test_mk_triple() {
        assert_eq!(mk_triple(0o751, Owner::User), "rwx");
        assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
        assert_eq!(mk_triple(0o751, Owner::Other), "--x");
        assert_eq!(mk_triple(0o600, Owner::Other), "---");
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(&0o755), "rwxr-xr-x");
        assert_eq!(format_mode(&0o421), "r---w---x");
    }
}
