use anyhow::Result;
use roan_engine::{context::Context, print_diagnostic};
use std::{fs::read_to_string, path::PathBuf};
use roan_engine::module::Module;
use roan_engine::source::Source;

pub fn run_command(file: String) -> Result<()> {
    let path = PathBuf::from(file);
    let content = read_to_string(&path)?;

    let ctx = Context::default();
    let source = Source::from_string(content.clone()).with_path(path);
    let module = Module::new(source);

    match ctx.eval(module) {
        Ok(_) => Ok(()),
        Err(e) => {
            print_diagnostic(e, Some(content));
            Ok(())
        }
    }
}
