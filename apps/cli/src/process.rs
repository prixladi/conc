use std::io::Error;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn execute_tail(files: Vec<String>) -> Error {
    Command::new("tail").arg("-f").args(files).exec()
}
