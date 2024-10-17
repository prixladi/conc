use std::io::Error;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn execute_tail(files: Vec<String>) -> Error {
    let n = if files.len() > 1 { 10 } else { 100 };

    Command::new("tail")
        .arg("-n")
        .arg(n.to_string())
        .arg("-f")
        .args(files)
        .exec()
}
