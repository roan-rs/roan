use anyhow::Result;
use dirs::home_dir;
use log::debug;
use std::{env::current_dir, fs, fs::create_dir, io::Cursor, path::PathBuf};

const TAR_BYTES: &'static [u8] = include_bytes!("../std.tar");

// pub fn prepare_home_dir() -> Result<PathBuf> {
//     let home_dir = home_dir().expect("Could not find home directory");
//     let roan_dir = home_dir.join(".roan");
//
//     if !roan_dir.exists() {
//         create_dir(&roan_dir)?;
//         debug!("Created roan directory at {:?}", roan_dir);
//     }
//
//     Ok(roan_dir)
// }

// Maybe simplify in the future
pub fn ensure_lib_dir() -> Result<(PathBuf, Vec<String>)> {
    let build_dir = current_dir()?.join("build");

    if !build_dir.exists() {
        create_dir(&build_dir)?;
        debug!("Created build directory at {:?}", build_dir);
    }

    let mut modules = vec![];
    let lib_dir = build_dir.join("std");
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

        modules.push(module_name);
    }

    debug!("Extracted std library to {:?}", lib_dir);

    Ok((lib_dir, modules))
}
