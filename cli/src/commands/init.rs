use crate::cli::opt;
use clap::{ArgAction, Command};

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
