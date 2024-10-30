use crate::context::GlobalContext;
use anyhow::Result;
use std::{fs, fs::create_dir, io::Cursor, path::PathBuf};
use tracing::debug;

const TAR_BYTES: &'static [u8] = include_bytes!("../std.tar");

// Maybe simplify in the future
pub fn ensure_lib_dir(global: &mut GlobalContext) -> Result<()> {
    let lib_dir = global.deps_dir()?.join("std");
    let cursor = Cursor::new(TAR_BYTES);

    let mut archive = tar::Archive::new(cursor);

    if lib_dir.exists() {
        fs::remove_dir_all(&lib_dir).expect("Could not remove old std directory");
    }

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let path = lib_dir.join(path);

        let module_name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        let mut target = lib_dir.to_owned().join(&module_name);
        for comp in path.components().skip(1) {
            target = target.join(comp);
        }

        if path.is_dir() {
            create_dir(&target)?;
        } else {
            fs::create_dir_all(target.parent().unwrap())?;
            let mut file = fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut file)?;
        }
    }

    debug!("Extracted std library to {:?}", lib_dir);

    Ok(())
}
