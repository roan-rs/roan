use std::{fs::File, path::PathBuf};
use tar::Builder;

fn main() {
    let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("std.tar");
    println!("cargo:warning=Building tarball at {:?}", full_path);
    let file = File::create(full_path).unwrap();
    let mut builder = Builder::new(file);

    let project_root = match project_root::get_project_root() {
        Ok(p) => p,
        Err(e) => panic!("Failed to get project root: {}", e),
    };
    let lib_path = project_root.join("lib");
    println!("cargo:warning=lib_path: {:?}", lib_path);

    builder.append_dir_all("", lib_path).unwrap();

    builder.finish().unwrap();

    println!("cargo:rerun-if-changed=std");
}
