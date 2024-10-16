use anyhow::Result;
use std::path::PathBuf;

fn remove_prefix(path: &PathBuf) -> PathBuf {
    let path_str = path.to_str().unwrap_or("");
    let normalized_str = if path_str.starts_with(r"\\?\") {
        &path_str[4..] // Strip the \\?\ prefix
    } else {
        path_str
    };
    PathBuf::from(normalized_str)
}

pub fn normalize_path(mut path: PathBuf, root: PathBuf) -> Result<PathBuf> {
    if path.is_relative() {
        path = root.join(path);
    }
    path = path.canonicalize()?;
    Ok(remove_prefix(&path))
}

pub fn canonicalize_path(path: PathBuf) -> Result<PathBuf> {
    let path = path.canonicalize()?;
    Ok(remove_prefix(&path))
}
