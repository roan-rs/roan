use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::Result;
use roan_engine::context::Context;
use roan_engine::print_diagnostic;

pub fn run_command(
    file: String
) -> Result<()> {
    let path = PathBuf::from(file);
    let content = read_to_string(&path)?;

    let ctx = Context::default();

    match ctx.eval() {
        Ok(_) => Ok(()),
        Err(e) => {
            print_diagnostic(e, Some(content));
            Ok(())
        }
    }
}
