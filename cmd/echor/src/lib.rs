use anyhow::Result;

#[derive(Debug)]
pub struct Options {
    pub text: Vec<String>,
    pub omit_newline: bool,
}

pub fn run(options: &Options) -> Result<()> {
    print!(
        "{}{}",
        options.text.join(" "),
        if options.omit_newline { "" } else { "\n" }
    );
    Ok(())
}
