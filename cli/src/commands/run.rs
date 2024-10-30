use crate::{cli::opt, context::GlobalContext, stds::ensure_lib_dir};
use anyhow::{anyhow, Result};
use clap::{ArgAction, ArgMatches, Command};
use colored::Colorize;
use roan_engine::{
    context::Context, module::Module, path::canonicalize_path, print_diagnostic, source::Source,
    vm::VM,
};
use std::fs::{create_dir, read_to_string};
use tracing::debug;

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

    let build_dir = global.build_dir()?;

    if !build_dir.exists() {
        create_dir(&build_dir)?;
        debug!("Created build directory at {:?}", build_dir);
    }

    ensure_lib_dir(global)?;

    let content = read_to_string(&path)?;

    let mut ctx = Context::builder().cwd(global.cwd.clone()).build();
    let source = Source::from_string(content.clone()).with_path(path);
    let vm = &mut VM::new();
    let mut module = Module::new(source);

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
