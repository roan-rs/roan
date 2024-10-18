use anyhow::{anyhow, Ok, Result};
use cli::cli;
use commands::run::run_command;
use logger::setup_logger;
use panic_handler::setup_panic_handler;

pub mod cli;
pub mod commands;
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

    let result = match cmd.0 {
        "run" => run_command(),
        _ => Err(anyhow!("Failed")),
    };

    match result {
        Ok => {
            log::debug!("Finished program")
        }
        Err(err) => {
            log::error!("{:?}", err);
        }
    }

    Ok(())
}
