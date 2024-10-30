use anstyle::Style;
use crate::{cli::opt, context::GlobalContext};
use anyhow::Result;
use clap::{ArgAction, Command};
use crate::style::WARN;

pub fn init_cmd() -> Command {
    Command::new("init")
        .about("Initialize a new project")
        .arg(
            opt("bin", "Create a binary project")
                .short('b')
                .action(ArgAction::SetTrue),
        )
        .arg(
            opt("lib", "Create a library project")
                .short('l')
                .action(ArgAction::SetTrue),
        )
}

pub fn init_command(ctx: &mut GlobalContext) -> Result<()> {

    Ok(())
}
