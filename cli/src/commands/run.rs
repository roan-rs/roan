use anyhow::Result;
use roan_engine::{context::Context, print_diagnostic, source::Source};
use std::{fs::read_to_string, path::PathBuf};

pub fn run_command(file: String) -> Result<()> {
    let path = PathBuf::from(file);
    let content = read_to_string(&path)?;

    let ctx = Context::default();
    let source = Source::from_string(content.clone(), Some(&path));
    println!("{:#?}", source);

    match ctx.eval() {
        Ok(_) => Ok(()),
        Err(e) => {
            print_diagnostic(e, Some(content));
            Ok(())
        }
    }
}
