use anyhow::{anyhow, bail, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let cmd = args.get(1).ok_or(anyhow!("No command provided"))?;
    let logs_dir = args.get(2).ok_or(anyhow!("No logs directory provided"))?;
    let logs_dir = env::current_dir()?.join(logs_dir);

    if !logs_dir.exists() {
        bail!("No logs directory found");
    }

    match cmd.as_str() {
        "clean" => {
            let entries = std::fs::read_dir(&logs_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                std::fs::remove_file(path)?;
            }

            println!("Logs cleaned up");
        }
        "list" => {
            let entries = std::fs::read_dir(&logs_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                println!("{}", path.display());
            }
        }
        _ => {
            println!("Invalid command");
        }
    }

    Ok(())
}
