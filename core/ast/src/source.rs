use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::PathBuf,
};
use std::str::Chars;
use log::debug;

/// A source of Roan code.
#[derive(Clone, Debug)]
pub struct Source {
    content: String,
    path: Option<PathBuf>,
}

impl Source {
    /// Creates a new `Source` from a `String`.
    pub fn from_string(string: String) -> Self {
        debug!("Creating source from string");
        Self {
            content: string,
            path: None,
        }
    }

    /// Creates a new `Source` from a byte slice.
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(source: &T) -> Self {
        debug!("Creating source from bytes");
        Self {
            content: source.as_ref().iter().map(|&b| b as char).collect(),
            path: None,
        }
    }

    /// Creates a new `Source` from a file path.
    pub fn from_path(path: PathBuf) -> io::Result<Self> {
        debug!("Creating source from path: {:?}", path);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            content: reader
                .bytes()
                .filter_map(|b| b.ok().map(|b| b as char))
                .collect(),
            path: Some(path),
        })
    }

    /// Sets or updates the path of this `Source`.
    pub fn with_path(self, new_path: PathBuf) -> Self {
        Self {
            content: self.content,
            path: Some(new_path),
        }
    }

    /// Returns the content of this `Source`.
    pub fn content(&self) -> String {
        self.content.clone()
    }

    /// Returns the path associated with this `Source`, if any.
    pub fn path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    /// Returns the content of this `Source`.
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Returns the content of this `Source` as a char iterator.
    pub fn chars(&self) -> Chars {
        self.content.chars()
    }

    /// Returns the content of this `Source` between the specified indices.
    pub fn get_between(&self, start: usize, end: usize) -> String {
        self.content[start..end].to_string()
    }
}

