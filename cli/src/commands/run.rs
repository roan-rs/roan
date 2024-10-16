use crate::std::ensure_lib_dir;
use anyhow::Result;
use clap::Command;
use roan_engine::{
    context::Context,
    module::Module,
    path::{canonicalize_path, normalize_path},
    print_diagnostic,
    source::Source,
    vm::VM,
};
use std::{fs::read_to_string, path::PathBuf};

pub fn run_cmd() -> Command {
    Command::new("run")
        .about("Run a project")
}

pub fn run_command() -> Result<()> {
    // let (lib_dir, modules) = ensure_lib_dir()?;

    // let path = normalize_path(PathBuf::from(file), std::env::current_dir()?)?;
    // let content = read_to_string(&path)?;

    // let ctx = Context::default();
    // let source = Source::from_string(content.clone()).with_path(path);
    // let vm = &mut VM::new();
    // let module = Module::new(source);

    // for mod_name in modules {
    //     let path = lib_dir.join(&mod_name).with_extension("roan");

    //     let content = read_to_string(&path)?;
    //     let source = Source::from_string(content.clone()).with_path(canonicalize_path(path)?);
    //     let module = Module::new(source);

    //     let module_name = format!("std::{}", mod_name);
    //     ctx.module_loader.insert(module_name, module);
    // }

    // match ctx.eval(module, vm) {
    //     Ok(_) => Ok(()),
    //     Err(e) => {
    //         print_diagnostic(e, Some(content));
    //         Ok(())
    //     }
    // }

    Ok(())
}
