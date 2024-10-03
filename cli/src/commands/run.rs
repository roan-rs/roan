use anyhow::Result;
use roan_engine::{context::Context, print_diagnostic};
use std::{fs::read_to_string, path::PathBuf};
use roan_engine::module::Module;
use roan_engine::source::Source;
use crate::std::{ensure_lib_dir};

pub fn run_command(file: String) -> Result<()> {
    let (lib_dir, modules) = ensure_lib_dir()?;

    let path = PathBuf::from(file);
    let content = read_to_string(&path)?;

    let ctx = Context::default();
    let source = Source::from_string(content.clone()).with_path(path);
    let module = Module::new(source);

    for mod_name in modules {
        let path = lib_dir.join(&mod_name).with_extension("roan");

        let content = read_to_string(&path)?;
        let source = Source::from_string(content.clone()).with_path(path);
        let module = Module::new(source);

        let module_name = format!("std::{}", mod_name);
        ctx.module_loader.insert(module_name, module);
    }

    match ctx.eval(module) {
        Ok(_) => Ok(()),
        Err(e) => {
            print_diagnostic(e, Some(content));
            Ok(())
        }
    }
}
