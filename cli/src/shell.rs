//! Mostly derived from [cargo](https://github.com/rust-lang/cargo) source code.

use crate::style::{ERROR, HEADER, NOTE, WARN};
use anstream::{AutoStream, ColorChoice};
use anstyle::Style;
use anyhow::Result;
use std::{
    fmt,
    io::{Stderr, Stdout, Write},
};

#[derive(Debug)]
pub struct ShellOutput {
    pub stdout: AutoStream<Stdout>,
    pub stderr: AutoStream<Stderr>,
    pub color: ColorChoice,
}

impl ShellOutput {
    pub fn stdout(&mut self) -> &mut AutoStream<Stdout> {
        &mut self.stdout
    }

    pub fn stderr(&mut self) -> &mut AutoStream<Stderr> {
        &mut self.stderr
    }
}

#[derive(Debug)]
pub struct Shell {
    pub output: ShellOutput,
}

impl Shell {
    pub fn new(color_choice: ColorChoice) -> Self {
        let output = ShellOutput {
            stdout: AutoStream::new(std::io::stdout(), color_choice),
            stderr: AutoStream::new(std::io::stderr(), color_choice),
            color: color_choice,
        };

        Self { output }
    }

    pub fn print(
        &mut self,
        status: &dyn fmt::Display,
        message: Option<&dyn fmt::Display>,
        style: &Style,
        justified: bool,
    ) -> Result<()> {
        let bold = anstyle::Style::new() | anstyle::Effects::BOLD;
        let dim = anstyle::Style::new() | anstyle::Effects::DIMMED;

        let mut buffer = Vec::new();
        if justified {
            write!(&mut buffer, "{style}{status:>15}{style:#}")?;
        } else {
            write!(&mut buffer, "{style}{status}{style:#}{dim}:{dim:#}")?;
        }
        match message {
            Some(message) => writeln!(buffer, " {message}")?,
            None => write!(buffer, " ")?,
        }
        self.output.stderr().write_all(&buffer)?;
        Ok(())
    }

    pub fn warn<T: fmt::Display>(&mut self, message: T) -> Result<()> {
        self.print(&"warning", Some(&message), &WARN, false)
    }

    pub fn note<T: fmt::Display>(&mut self, message: T) -> Result<()> {
        self.print(&"note", Some(&message), &NOTE, false)
    }

    pub fn error<T: fmt::Display>(&mut self, message: T) -> Result<()> {
        self.print(&"error", Some(&message), &ERROR, false)
    }

    pub fn status<T, U>(&mut self, status: T, message: U) -> Result<()>
    where
        T: fmt::Display,
        U: fmt::Display,
    {
        self.print(&status, Some(&message), &HEADER, true)
    }

    pub fn set_color_choice(&mut self, color_choice: ColorChoice) {
        let (stdout, stderr, color) = (
            &mut self.output.stdout,
            &mut self.output.stderr,
            &mut self.output.color,
        );

        *color = color_choice;
        *stdout = AutoStream::new(std::io::stdout(), color_choice);
        *stderr = AutoStream::new(std::io::stderr(), color_choice);
    }
}
