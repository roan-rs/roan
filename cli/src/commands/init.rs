use crate::{
    cli::{opt, positional},
    context::GlobalContext,
};
use anyhow::{bail, Result};
use clap::{ArgAction, ArgMatches, Command};
use std::{fmt::Display, fs, process};

pub fn init_cmd() -> Command {
    Command::new("init")
        .about("Initialize a new project")
        .arg(positional("name", "The name of the project"))
        .arg(
            opt("bin", "Create a binary project")
                .short('b')
                .action(ArgAction::SetTrue),
        )
        .arg(
            opt("lib", "Create a library project")
                .short('l')
                .action(ArgAction::SetTrue),
        )
        .arg(
            opt(
                "force",
                "Force initialization even if the directory is not empty",
            )
            .short('f')
            .action(ArgAction::SetTrue),
        )
        .arg(
            opt("no-git", "Do not initialize git repository")
                .long("no-git")
                .action(ArgAction::SetTrue),
        )
}

#[derive(Debug, Clone)]
pub enum ProjectType {
    Bin,
    Lib,
}

impl Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Bin => write!(f, "binary"),
            ProjectType::Lib => write!(f, "library"),
        }
    }
}

pub fn init_command(ctx: &mut GlobalContext, args: &ArgMatches) -> Result<()> {
    let name = args.get_one::<String>("name");

    if name.is_none() {
        bail!("Project name is required");
    }
    let name = name.unwrap().clone();

    let project_type = match (args.get_flag("bin"), args.get_flag("lib")) {
        (true, false) => ProjectType::Bin,
        (false, true) => ProjectType::Lib,
        (false, false) => ProjectType::Bin,
        (true, true) => bail!("Cannot create both binary and library project"),
    };
    let force = args.get_flag("force");

    let project_dir = ctx.cwd.join(name.clone());

    if project_dir.exists() {
        if force {
            ctx.shell.warn("Force flag is enabled")?;
            fs::remove_dir_all(project_dir)?;
        } else {
            bail!("Project directory already exists");
        }
    }

    ctx.shell
        .status("Creating", format!("{} project", project_type))?;

    if name == "std" {
        ctx.shell
            .warn("'std' is a part of the standard library and it is recommended to not use it as a project name")?;
    }

    if name.chars().any(|ch| ch > '\x7f') {
        ctx.shell
            .warn("Project name contains non-ascii characters")?;
    }

    let project_dir = ctx.cwd.join(name.clone());

    std::fs::create_dir(&project_dir)?;

    if !args.get_flag("no-git") {
        init_git(ctx, &project_dir)?;
        create_gitignore(ctx, &project_dir)?;
    }

    create_roan_toml(ctx, &project_dir, &name, project_type.clone())?;
    create_source_files(ctx, &project_dir, project_type)?;

    Ok(())
}

const GITIGNORE: &str = r#"# Logs

logs

# Coverage directory used by tools like istanbul

coverage
*.lcov

# dotenv environment variable files

.env
.env.development.local
.env.test.local
.env.production.local
.env.local

# Stores VSCode versions used for testing VSCode extensions

.vscode-test

# IntelliJ based IDEs
.idea

# Finder (MacOS) folder config
.DS_Store

# Build folder
build
"#;

fn create_gitignore(ctx: &mut GlobalContext, project_dir: &std::path::Path) -> Result<()> {
    let gitignore = project_dir.join(".gitignore");

    ctx.shell.status("Creating", ".gitignore")?;
    std::fs::write(&gitignore, GITIGNORE)?;

    Ok(())
}

fn init_git(ctx: &mut GlobalContext, project_dir: &std::path::Path) -> Result<()> {
    ctx.shell.status("Initializing", "git repository")?;

    let output = process::Command::new("git")
        .arg("init")
        .current_dir(project_dir)
        .output()?;

    if !output.status.success() {
        ctx.shell.error(format!(
            "Failed to initialize git repository: {}",
            String::from_utf8_lossy(&output.stderr)
        ))?;
    }

    Ok(())
}

fn create_roan_toml(
    ctx: &mut GlobalContext,
    project_dir: &std::path::Path,
    name: &str,
    project_type: ProjectType,
) -> Result<()> {
    let r#type = match project_type {
        ProjectType::Bin => "bin",
        ProjectType::Lib => "lib",
    };

    let mut file = toml_edit::DocumentMut::new();

    file["project"] = toml_edit::Item::Table(toml_edit::Table::default());
    file["project"]["name"] = toml_edit::value(name);
    file["project"]["version"] = toml_edit::value("0.1.0");
    file["project"]["type"] = toml_edit::value(r#type);
    file["dependencies"] = toml_edit::Item::Table(toml_edit::Table::default());

    let mut std_dep = toml_edit::InlineTable::default();
    std_dep.insert("version", toml_edit::value("0.1.0").into_value().unwrap());
    std_dep.insert(
        "github",
        toml_edit::value("roan-rs/std").into_value().unwrap(),
    );
    file["dependencies"]["std"] = toml_edit::Item::Value(toml_edit::Value::InlineTable(std_dep));

    let toml = file.to_string();

    let roan_toml = project_dir.join("roan.toml");

    ctx.shell.status("Creating", "roan.toml")?;

    fs::write(&roan_toml, toml)?;

    Ok(())
}

const BIN_CONTENT: &str = r#"
use { println } from "std::debug";

fn main() {
    println("Hello, {}!", "universe");
}

main();
"#;

const LIB_CONTENT: &str = r#"
pub fn add(a: int, b: int) -> int {
    a + b
}
"#;

fn create_source_files(
    ctx: &mut GlobalContext,
    project_dir: &std::path::Path,
    project_type: ProjectType,
) -> Result<()> {
    let src_dir = project_dir.join("src");
    ctx.shell.status("Creating", "source files")?;
    std::fs::create_dir(&src_dir)?;

    let (content, file) = match project_type {
        ProjectType::Bin => (BIN_CONTENT, src_dir.join("main.roan")),
        ProjectType::Lib => (LIB_CONTENT, src_dir.join("lib.roan")),
    };

    fs::write(&file, content)?;
    Ok(())
}
