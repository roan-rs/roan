use crate::{commands::run::run_cmd, style};
use clap::{builder::Styles, Arg, ArgAction, Command};

pub fn opt(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help).action(ArgAction::Set)
}

pub fn cli() -> Command {
    let styles = {
        Styles::styled()
            .header(style::HEADER)
            .usage(style::USAGE)
            .literal(style::LITERAL)
            .placeholder(style::PLACEHOLDER)
            .error(style::ERROR)
            .valid(style::VALID)
            .invalid(style::INVALID)
    };

    Command::new("roan")
        .allow_external_subcommands(true)
        .styles(styles)
        .arg(
            opt("verbose", "Use verbose output")
                .short('v')
                .action(ArgAction::SetTrue)
                .global(true),
        )
        .subcommand(run_cmd())
}
