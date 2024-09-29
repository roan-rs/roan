use std::path::PathBuf;
use anyhow::Result;
use roan_engine::print_diagnostic;
use crate::instance::Instance;

pub fn run_command(
    file: String
) -> Result<()> {
    let path = PathBuf::from(file);
    let mut project = Instance::from_path(path);

    match project.interpret() {
        Ok(_) => {
            log::debug!("Ran file");
            Ok(())
        }
        Err(err) => {
            print_diagnostic(err, Some(project.content));
            Ok(())
        }
    }
}
