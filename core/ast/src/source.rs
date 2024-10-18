use tracing::debug;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::PathBuf,
    str::Chars,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_from_string() {
        let source = Source::from_string("fn main() {}".to_string());

        assert_eq!(source.content(), "fn main() {}");
        assert_eq!(source.path(), None);
    }

    #[test]
    fn test_source_from_bytes() {
        let source = Source::from_bytes(b"fn main() {}");

        assert_eq!(source.content(), "fn main() {}");
        assert_eq!(source.path(), None);
    }

    #[test]
    fn test_source_with_path() {
        let source = Source::from_string("fn main() {}".to_string())
            .with_path(PathBuf::from("tests/test.roan"));

        assert_eq!(source.content(), "fn main() {}");
        assert_eq!(source.path(), Some(PathBuf::from("tests/test.roan")));
    }

    #[test]
    fn test_source_len() {
        let source = Source::from_string("fn main() {}".to_string());

        assert_eq!(source.len(), 12);
    }

    #[test]
    fn test_source_chars() {
        let source = Source::from_string("fn main() {}".to_string());

        assert_eq!(source.chars().collect::<String>(), "fn main() {}");
    }

    #[test]
    fn test_source_get_between() {
        let source = Source::from_string("fn main() {}".to_string());

        assert_eq!(source.get_between(3, 7), "main");
    }
}
