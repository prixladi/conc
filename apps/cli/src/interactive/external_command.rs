use std::{
    io::{self, stdout, Stdout},
    process::Command,
};

use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::CrosstermBackend, Terminal};

#[derive(Debug, thiserror::Error)]
pub enum ExternalCommandError {
    #[error("IO error occurred while executing external command: {inner}")]
    IO { inner: io::Error },
    #[error("Unable to determine command for action '{action}', check your application config")]
    MissingCommand { action: String },
}

impl From<io::Error> for ExternalCommandError {
    fn from(value: io::Error) -> Self {
        Self::IO { inner: value }
    }
}

pub(super) fn open_log_file(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    viewer: &[String],
    path: String,
) -> Result<(), ExternalCommandError> {
    stdout().execute(LeaveAlternateScreen)?;

    let command_str = viewer.first().ok_or(ExternalCommandError::MissingCommand {
        action: String::from("open_log_file"),
    })?;

    let mut command = Command::new(command_str);
    for arg in viewer.iter().skip(1) {
        command.arg(arg);
    }
    command.arg(path);
    command.status()?;

    stdout().execute(EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

pub(super) fn open_string_in_less(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    str: String,
) -> std::io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    Command::new("bash")
        .arg("-c")
        .arg(format!("echo \'{}\' | less", str))
        .status()?;

    stdout().execute(EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}
