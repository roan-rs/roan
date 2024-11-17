use crate::{
    commands::{init::init_command, install::install_command},
    context::GlobalContext,
};
use anstream::ColorChoice;
use anyhow::Result;
use clap::ArgMatches;
use cli::cli;
use commands::run::run_command;
use logger::setup_tracing;
use panic_handler::setup_panic_handler;
use roan_engine::print_diagnostic;
use std::{env, process::exit};

pub mod cli;
pub mod commands;
mod config_file;
mod context;
mod fs;
pub mod logger;
mod module_loader;
pub mod panic_handler;
pub mod pm;

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic_handler();
    let args = cli().try_get_matches().unwrap_or_else(|err| {
        err.print().expect("Error printing error");
        exit(1);
    });
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

    match run_cmd(&mut ctx, cmd).await {
        Ok(()) => Ok(()),
        Err(err) => {
            if let None = print_diagnostic(&err, None, None) {
                ctx.shell.error(&format!("{}", err))?;
            }

            exit(1);
        }
    }
}

pub async fn run_cmd(ctx: &mut GlobalContext, cmd: (&str, &ArgMatches)) -> Result<()> {
    match cmd.0 {
        "run" => run_command(ctx, cmd.1),
        "init" => init_command(ctx, cmd.1),
        "install" => install_command(ctx, cmd.1).await,
        _ => {
            cli().print_help()?;
            exit(1);
        }
    }
}
