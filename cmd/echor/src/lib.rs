use std::io::Write;

use anyhow::Result;

#[derive(Debug)]
pub struct Options {
    pub text: Vec<String>,
    pub omit_newline: bool,
}

pub fn run(writer: &mut impl Write, options: &Options) -> Result<()> {
    write!(
        writer,
        "{}{}",
        options.text.join(" "),
        if options.omit_newline { "" } else { "\n" }
    )?;
    Ok(())
}
