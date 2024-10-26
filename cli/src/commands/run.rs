use crate::{cli::opt, context::GlobalContext, stds::ensure_lib_dir};
use anyhow::{anyhow, Result};
use clap::{ArgAction, ArgMatches, Command};
use colored::Colorize;
use roan_engine::{
    context::Context, module::Module, path::canonicalize_path, print_diagnostic, source::Source,
    vm::VM,
};
use std::fs::read_to_string;

pub fn run_cmd() -> Command {
    Command::new("run").about("Run a project").arg(
        opt("time", "Prints the time taken to run the project")
            .short('t')
            .action(ArgAction::SetTrue),
    )
}

pub fn run_command(global: &mut GlobalContext, matches: &ArgMatches) -> Result<()> {
    global.load_config()?;
    let path = global.get_main_file()?;

    if global.project_type()? == "lib" {
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
        Ok(_) => {}
        Err(e) => {
            print_diagnostic(e, Some(content));
            std::process::exit(1);
        }
    }

    if matches.get_flag("time") {
        println!(
            "Finished program in: {}",
            format!("{:?}", global.start.elapsed()).cyan()
        );
    }

    Ok(())
}
