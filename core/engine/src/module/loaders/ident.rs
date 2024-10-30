use std::path::PathBuf;

#[derive(Debug)]
pub struct ModuleIdentifier {
    pub main_name: String,
    pub rest: Option<Vec<String>>,
}

impl ModuleIdentifier {
    /// File name is always the last part of the identifier. If only one part is present file name defaults to `lib.roan`
    pub fn file_name(&self) -> String {
        match &self.rest {
            Some(rest) => rest.join(std::path::MAIN_SEPARATOR_STR) + ".roan",
            None => "lib.roan".to_string(),
        }
    }

    /// Checks if spec is a module identifier and parses it. Module identifier consists of a main name and a sub name separated by a `::`.
    ///
    /// "std::io" -> build/deps/std/io.roan
    ///
    /// "std::io::file" -> build/deps/std/io/file.roan
    ///
    /// "some_dep" -> build/deps/some_dep/lib.roan
    ///
    /// "std" -> build/deps/std/lib.roan
    ///
    /// # Arguments
    ///
    /// * `spec` - A string slice that represents the specification of the path to check.
    ///
    /// # Returns
    ///
    /// A boolean value indicating whether the spec is a module identifier.
    pub fn parse_module_identifier(spec: &str) -> Option<ModuleIdentifier> {
        let parts: Vec<&str> = spec.split("::").collect();
        if parts.len() == 1 {
            Some(ModuleIdentifier {
                main_name: parts[0].to_string(),
                rest: None,
            })
        } else if parts.len() >= 2 {
            Some(ModuleIdentifier {
                main_name: parts[0].to_string(),
                rest: Some(parts[1..].iter().map(|s| s.to_string()).collect()),
            })
        } else {
            None
        }
    }
}
