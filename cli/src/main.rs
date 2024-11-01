use crate::{
    commands::{init::init_command, install::install_command},
    context::GlobalContext,
};
use anstream::ColorChoice;
use anyhow::{Ok, Result};
use cli::cli;
use commands::run::run_command;
use logger::setup_tracing;
use panic_handler::setup_panic_handler;
use std::{env, process::exit};
use tracing_subscriber::fmt::format;

pub mod cli;
pub mod commands;
mod config_file;
mod context;
mod fs;
pub mod logger;
pub mod panic_handler;
pub mod pm;
pub mod shell;
pub mod stds;
pub mod style;

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic_handler();
    let args = cli().try_get_matches()?;
    let verbose = args.get_flag("verbose");

    env::set_var("ROAN_LOG", if verbose { "trace" } else { "info" });
    setup_tracing(verbose);

    tracing::debug!("Starting roan-cli");

    let cmd = match args.subcommand() {
        Some((cmd, args)) => (cmd, args),
        None => {
            cli().print_help()?;

            return Ok(());
        }
    };

    let color_choice = if args.get_flag("no-color") {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };

    let mut ctx = GlobalContext::default(color_choice)?;
    ctx.verbose = verbose;

    if let Err(err) = match cmd.0 {
        "run" => run_command(&mut ctx, cmd.1),
        "init" => init_command(&mut ctx, cmd.1),
        "install" => install_command(&mut ctx, cmd.1).await,
        _ => {
            cli().print_help()?;
            exit(1);
        }
    } {
        ctx.shell.error(format!("{:?}", err))?;

        Ok(())
    } else {
        Ok(())
    }
}
