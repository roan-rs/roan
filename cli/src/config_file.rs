use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub struct RoanConfig {
    pub project: ProjectConfig,
    pub tasks: Option<HashMap<String, String>>,
    pub dependencies: Option<HashMap<String, Dependency>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub r#type: Option<String>,
    pub lib: Option<PathBuf>,
    pub bin: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dependency {
    pub version: String,
    pub path: Option<String>,
    pub git: Option<String>,
}