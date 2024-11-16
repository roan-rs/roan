use crate::{
    cli::opt,
    context::GlobalContext,
    pm::{
        packs::{parse_pack, PackVersion},
        source::PackageSource,
    },
};
use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use itertools::Itertools;

pub fn install_cmd() -> Command {
    Command::new("install")
        .arg(
            Arg::new("packs")
                .value_name("PACK[@<VER>]")
                .help("Select the package from the given source")
                .value_parser(parse_pack)
                .num_args(0..),
        )
        .arg(
            opt("git", "Git URL to install the specified crate from")
                .value_name("URL")
                .conflicts_with_all(&["path"]),
        )
        .arg(
            opt("branch", "Branch to use when installing from git")
                .value_name("BRANCH")
                .requires("git"),
        )
        .arg(
            opt("tag", "Tag to use when installing from git")
                .value_name("TAG")
                .requires("git"),
        )
        .arg(
            opt("rev", "Specific commit to use when installing from git")
                .value_name("SHA")
                .requires("git"),
        )
        .arg(
            opt("path", "Filesystem path to local crate to install from")
                .value_name("PATH")
                .conflicts_with_all(&["git"]),
        )
}

pub async fn install_command(_: &mut GlobalContext, matches: &ArgMatches) -> Result<()> {
    let packages = matches
        .get_many::<PackVersion>("packs")
        .unwrap_or_default()
        .cloned()
        .dedup_by(|a, b| a == b)
        .collect::<Vec<_>>();

    let _source = PackageSource::from_arg_matches(matches)?;

    println!("Installing packages: {:?}", packages);

    Ok(())
}
