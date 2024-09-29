use anyhow::Result;
use clap::{
    builder::{styling, PossibleValuesParser, Styles, TypedValueParser},
    Args, Parser, Subcommand, ValueHint,
};
use cli::{Cli, Commands};
use commands::run::run_command;
use logger::setup_logger;
use panic_handler::setup_panic_handler;

pub mod cli;
pub mod commands;
pub mod logger;
pub mod panic_handler;


fn main() -> Result<()> {
    setup_panic_handler();
    let args = Cli::parse();
    setup_logger(args.verbose);

    log::debug!("Parsed clap arguments");

    let result = match args.command {
        Commands::Run {
            file
        } => run_command(file),
    };

    match result {
        Ok(_) => {
            log::debug!("Finished program")
        }
        Err(err) => {
            log::error!("{:?}", err);
        }
    }

    Ok(())
}
