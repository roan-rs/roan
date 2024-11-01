use crate::{cli::opt, context::GlobalContext, pm::entry::InstallEntry};
use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use std::{error::Error, fs};

pub fn install_cmd() -> Command {
    Command::new("install").arg(
        Arg::new("package")
            .help("The packages to install")
            .required(false)
            .num_args(0..256),
    )
}

pub async fn install_command(ctx: &mut GlobalContext, matches: &ArgMatches) -> Result<()> {
    let packages: Vec<String> = matches
        .get_many::<String>("package")
        .expect("No packages to install")
        .map(|s| s.to_string())
        .collect();

    fs::create_dir_all(ctx.cache_dir()?)?;
    
    ctx.load_config()?;

    for package in packages {
        let entry = InstallEntry::from_string(package)?;

        match ctx.install(entry).await {
            Ok(_) => {}
            Err(err) => {
                let err_msg = format!("{:?}", err);
                if let Ok(err) = err.downcast::<octocrab::Error>() {
                    ctx.shell
                        .error(format!("{}", err.source().unwrap().to_string()))?;
                } else {
                    ctx.shell.error(err_msg)?;
                }
            }
        }
    }

    Ok(())
}
