use crate::{context::GlobalContext, stds::ensure_lib_dir};
use anyhow::{anyhow, Result};
use clap::Command;
use roan_engine::{
    context::Context, module::Module, path::canonicalize_path, print_diagnostic, source::Source,
    vm::VM,
};
use std::fs::read_to_string;

pub fn run_cmd() -> Command {
    Command::new("run").about("Run a project")
}

pub fn run_command(ctx: &mut GlobalContext) -> Result<()> {
    ctx.load_config()?;
    let path = ctx.get_main_file()?;

    if ctx.project_type()? == "lib" {
        return Err(anyhow!("Cannot run a library project."));
    }

    let (lib_dir, modules) = ensure_lib_dir()?;
    let content = read_to_string(&path)?;

    let mut ctx = Context::default();
    let source = Source::from_string(content.clone()).with_path(path);
    let vm = &mut VM::new();
    let mut module = Module::new(source);

    for mod_name in modules {
        let path = lib_dir.join(&mod_name).with_extension("roan");

        let content = read_to_string(&path)?;
        let source = Source::from_string(content.clone()).with_path(canonicalize_path(path)?);
        let module = Module::new(source);

        let module_name = format!("std::{}", mod_name);
        ctx.insert_module(module_name, module);
    }

    match ctx.eval(&mut module, vm) {
        Ok(_) => Ok(()),
        Err(e) => Ok({
            print_diagnostic(e, Some(content));
        }),
    }
}
