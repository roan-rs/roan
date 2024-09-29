use std::{
    fs::File,
    io::{self, BufReader, Bytes, Cursor, Read},
    path::Path,
};

/// A trait for reading Unicode characters from a source.
pub trait ReadChar {
    /// Retrieves the next character as a `char`.
    ///
    /// Returns `Ok(Some(char))` if a character is successfully read,
    /// `Ok(None)` if the end of the input is reached,
    /// or an `io::Error` if an I/O error occurs.
    fn next_char(&mut self) -> io::Result<Option<char>>;
}

/// UTF-8 encoded input source.
#[derive(Debug)]
pub struct UTF8Input<R> {
    input: Bytes<R>,
}

impl<R: Read> UTF8Input<R> {
    /// Creates a new `UTF8Input` from the given reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader implementing the `Read` trait.
    pub fn new(reader: R) -> Self {
        Self {
            input: reader.bytes(),
        }
    }

    /// Retrieves the next byte from the input.
    fn next_byte(&mut self) -> io::Result<Option<u8>> {
        self.input.next().transpose()
    }
}

impl<R: Read> ReadChar for UTF8Input<R> {
    /// Retrieves the next character as a `char`.
    ///
    /// This method decodes UTF-8 byte sequences into Unicode scalar values.
    /// It handles multi-byte sequences and ensures that only valid UTF-8
    /// sequences are converted to `char`. Invalid sequences result in an error.
    fn next_char(&mut self) -> io::Result<Option<char>> {
        let first_byte = match self.next_byte()? {
            Some(b) => b,
            None => return Ok(None),
        };

        let char_result = match first_byte {
            0x00..=0x7F => {
                // Single-byte (ASCII) character
                Ok(first_byte as char)
            }
            0xC0..=0xDF => {
                // Two-byte sequence
                let second_byte = self.next_byte()?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of input")
                })?;
                let code_point = ((first_byte & 0x1F) as u32) << 6 | (second_byte & 0x3F) as u32;
                std::char::from_u32(code_point).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence")
                })
            }
            0xE0..=0xEF => {
                // Three-byte sequence
                let second_byte = self.next_byte()?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of input")
                })?;
                let third_byte = self.next_byte()?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of input")
                })?;
                let code_point = ((first_byte & 0x0F) as u32) << 12
                    | ((second_byte & 0x3F) as u32) << 6
                    | (third_byte & 0x3F) as u32;
                std::char::from_u32(code_point).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence")
                })
            }
            0xF0..=0xF7 => {
                // Four-byte sequence
                let second_byte = self.next_byte()?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of input")
                })?;
                let third_byte = self.next_byte()?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of input")
                })?;
                let fourth_byte = self.next_byte()?.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of input")
                })?;
                let code_point = ((first_byte & 0x07) as u32) << 18
                    | ((second_byte & 0x3F) as u32) << 12
                    | ((third_byte & 0x3F) as u32) << 6
                    | (fourth_byte & 0x3F) as u32;
                std::char::from_u32(code_point).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 sequence")
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid UTF-8 start byte",
            )),
        };

        // Correctly map the Result<char, io::Error> to Result<Option<char>, io::Error>
        char_result.map(Some)
    }
}

impl<R: Read> Iterator for UTF8Input<R> {
    type Item = io::Result<char>;

    /// Advances the iterator and returns the next character.
    ///
    /// This method allows `UTF8Input` to be used in iterator contexts.
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_char() {
            Ok(Some(c)) => Some(Ok(c)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// A source of Roan code.
///
/// The `Source` struct abstracts over different kinds of input sources,
/// such as files, strings, and byte slices, providing a unified interface
/// for reading characters.
///
/// # Type Parameters
///
/// * `'path` - The lifetime of the path reference, if any.
/// * `R` - The underlying reader type.
#[derive(Debug)]
pub struct Source<'path, R> {
    reader: R,
    path: Option<&'path Path>,
}

impl<'path, R: Read> Source<'path, UTF8Input<R>> {
    /// Creates a new `Source` from a reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader implementing the `Read` trait.
    /// * `path` - An optional file path associated with the source.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::Cursor;
    /// use roan_engine::source::Source;
    ///
    /// let cursor = Cursor::new("Hello, world!");
    /// let source = Source::from_reader(cursor, None);
    /// ```
    pub fn from_reader(reader: R, path: Option<&'path Path>) -> Self {
        Self {
            reader: UTF8Input::new(reader),
            path,
        }
    }
}

impl<'path> Source<'path, UTF8Input<Cursor<String>>> {
    /// Creates a new `Source` from a `String`.
    ///
    /// # Arguments
    ///
    /// * `string` - The input string.
    /// * `path` - An optional file path associated with the source.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roan_engine::source::Source;
    /// let source = Source::from_string("Hello, world!".to_string(), None);
    /// ```
    pub fn from_string(string: String, path: Option<&'path Path>) -> Self {
        Self {
            reader: UTF8Input::new(Cursor::new(string)),
            path,
        }
    }
}

impl<'bytes> Source<'static, UTF8Input<&'bytes [u8]>> {
    /// Creates a new `Source` from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `source` - A reference to a byte slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use roan_engine::source::Source;
    /// let bytes = b"Hello, byte slice!";
    /// let source = Source::from_bytes(&bytes[..]);
    /// ```
    pub fn from_bytes<T: AsRef<[u8]> + ?Sized>(source: &'bytes T) -> Self {
        Self {
            reader: UTF8Input::new(source.as_ref()),
            path: None,
        }
    }
}

impl<'path> Source<'path, UTF8Input<BufReader<File>>> {
    /// Creates a new `Source` from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to be opened.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the file cannot be opened.
    ///
    /// # Examples
    ///
    /// ```rust no_run
    /// use std::path::Path;
    /// use roan_engine::source::Source;
    ///
    /// let path = Path::new("example.txt");
    /// let source = Source::from_path(path).expect("Failed to open file");
    /// ```
    pub fn from_path(path: &'path Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader: UTF8Input::new(reader),
            path: Some(path),
        })
    }
}

impl<'path, R> Source<'path, R> {
    /// Sets or updates the path of this `Source`.
    ///
    /// This method consumes the current `Source` and returns a new one with
    /// the updated path.
    ///
    /// # Arguments
    ///
    /// * `new_path` - The new path to associate with the source.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use roan_engine::source::Source;
    ///
    /// let source = Source::from_bytes(&b"Hello"[..]);
    /// let new_path = Path::new("new_path.txt");
    /// let updated_source = source.with_path(new_path);
    /// ```
    pub fn with_path(self, new_path: &Path) -> Source<'_, R> {
        Source {
            reader: self.reader,
            path: Some(new_path),
        }
    }

    /// Returns the path associated with this `Source`, if any.
    ///
    /// # Examples
    ///
    /// ```rust no_run
    /// use std::path::Path;
    /// use roan_engine::source::Source;
    ///
    /// let path = Path::new("example.txt");
    /// let source = Source::from_path(path).expect("Failed to open file");
    /// assert_eq!(source.path(), Some(path));
    /// ```
    pub fn path(&self) -> Option<&'path Path> {
        self.path
    }
}

impl<'path, R: Read> ReadChar for Source<'path, UTF8Input<R>> {
    /// Retrieves the next character from the source.
    ///
    /// This method delegates to the underlying `UTF8Input`'s `next_char` method.
    fn next_char(&mut self) -> io::Result<Option<char>> {
        self.reader.next_char()
    }
}

impl<'path, R: Read> Iterator for Source<'path, UTF8Input<R>> {
    type Item = io::Result<char>;

    /// Advances the iterator and returns the next character from the source.
    ///
    /// This allows `Source` to be used in iterator contexts, such as loops.
    fn next(&mut self) -> Option<Self::Item> {
        self.reader.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_input() {
        let input = Cursor::new("Hello");
        let mut utf8_input = UTF8Input::new(input);

        let mut chars = Vec::new();
        while let Ok(Some(c)) = utf8_input.next_char() {
            chars.push(c);
        }

        assert_eq!(chars, vec!['H', 'e', 'l', 'l', 'o']);
    }

    #[test]
    fn test_multibyte_input() {
        let input = Cursor::new("你好"); // "Hello" in Chinese
        let mut utf8_input = UTF8Input::new(input);

        let mut chars = Vec::new();
        while let Ok(Some(c)) = utf8_input.next_char() {
            chars.push(c);
        }

        assert_eq!(chars, vec!['你', '好']);
    }

    #[test]
    fn test_invalid_utf8() {
        let input = Cursor::new(&[0xFF, 0xFF, 0xFF]);
        let mut utf8_input = UTF8Input::new(input);

        match utf8_input.next_char() {
            Err(e) => assert_eq!(e.kind(), io::ErrorKind::InvalidData),
            _ => panic!("Expected an InvalidData error"),
        }
    }
}
