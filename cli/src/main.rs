use crate::context::GlobalContext;
use anyhow::{anyhow, Ok, Result};
use cli::cli;
use commands::run::run_command;
use logger::setup_logger;
use panic_handler::setup_panic_handler;

pub mod cli;
pub mod commands;
mod config_file;
mod context;
mod fs;
pub mod logger;
pub mod panic_handler;
mod std;
pub mod style;

fn main() -> Result<()> {
    setup_panic_handler();
    let args = cli().try_get_matches()?;
    let verbose = args.get_flag("verbose");

    setup_logger(verbose);

    log::debug!("Parsed clap arguments");

    let cmd = match args.subcommand() {
        Some((cmd, args)) => (cmd, args),
        None => {
            cli().print_help()?;

            return Ok(());
        }
    };

    let mut ctx = GlobalContext::default()?;
    ctx.verbose = verbose;

    let result = match cmd.0 {
        "run" => run_command(&mut ctx),
        _ => {
            cli().print_help()?;
            
            Err(anyhow!("Unknown command"))
        },
    };

    match result {
        Err(err) => {
            log::error!("{:?}", err);
        }
        _ok => {},
    }

    Ok(())
}
