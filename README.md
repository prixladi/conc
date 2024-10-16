# Conc (WIP)

Conc is a simple process manager.

## Architecture

The core of the application (daemon) is written in C, and supporting tools (CLI, GUI, TUI) are written in Rust.

### Languages

`/` - Rust

`/apps/daemon` - C

### Folder structure

    .
    ├── apps
    │   ├── daemon              # Systemd daemon handling the core of the process management
    │   ├── cli                 # Command line interface communication with the daemon
    │   ├── gui                 # Graphical interface communicating with daemon (Planned)
    │   └── tui                 # Terminal interface communicating with daemon (Planned)
    ├── crates
    │   ├── daemon-client       # Shared Rust library that provides a thin layer for communication with the daemon
    │   └── project-settings    # Shared Rust library for searching and parsing project settings
    └── Cargo.toml

## Installation

There is currently no distribution of binaries so you need to compile the project yourself. For the installation process to work, you also need to have `systemd` installed and running. The application does not rely on `systemd` in any way but you will need to run the installed daemon in some other way if you are not using `systemd`.

### Requirements

- [make](https://www.gnu.org/software/make/) - to orchestrate the installation process
- [gcc](https://gcc.gnu.org/) - to compile the daemon
- [cargo](https://github.com/rust-lang/cargo) - to compile the cli

### Unix like

Run `make install` to install the daemon and CLI and then run `sudo systemctl start concd` to start the daemon. This will install `concd` as a system service and `concc` as a binary that can be used from the command line `conc -h`.

### Other

The project is currently heavily dependent on many unix-like features like `unix socket` and `unistd.h` in libc, so there is no way to install **conc** on other systems.
