use crate::{cli::opt, context::GlobalContext, module_loader::RoanModuleLoader};
use anyhow::Result;
use clap::{ArgAction, ArgMatches, Command};
use colored::Colorize;
use roan_engine::{context::Context, module::Module, print_diagnostic, source::Source, vm::VM};
use std::{
    cell::RefCell,
    fs::{create_dir, read_to_string},
    process::exit,
    rc::Rc,
};
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

    global
        .shell
        .status("Running", &path.display().to_string())?;

    global.assert_type("bin")?;

    let build_dir = global.build_dir()?;

    if !build_dir.exists() {
        create_dir(&build_dir)?;
        debug!("Created build directory at {:?}", build_dir);
    }

    let content = read_to_string(&path)?;

    let ctx = &mut Context::builder()
        .cwd(global.cwd.clone())
        .module_loader(Rc::new(RefCell::new(RoanModuleLoader::new())))
        .build();
    let source = Source::from_string(content.clone()).with_path(path);
    let vm = &mut VM::new();
    let mut module = Module::new(source);

    let result: Result<(), anyhow::Error> = {
        let parse_start = std::time::Instant::now();

        match module.parse(ctx, vm) {
            Ok(..) => {}
            Err(err) => {
                print_diagnostic(&err, Some(content), module.path());
                exit(1);
            }
        }

        global.shell.status(
            "Finished",
            format!("parsing in {:?}", parse_start.elapsed()),
        )?;

        module.interpret(ctx, vm)?;

        Ok(())
    };

    match result {
        Ok(_) => {}
        Err(e) => {
            print_diagnostic(&e, Some(content), module.path());
            exit(1);
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
