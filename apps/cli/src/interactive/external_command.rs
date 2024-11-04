use std::{
    io::{stdout, Stdout},
    process::Command,
};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::CrosstermBackend, Terminal};

pub(super) fn open_log_file_in_less(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    path: String,
) -> std::io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Command::new("less").arg("+GF").arg(path).status()?;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}

pub(super) fn open_string_in_less(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    str: String,
) -> std::io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Command::new("bash")
        .arg("-c")
        .arg(format!("echo \'{}\' | less", str))
        .status()?;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}
