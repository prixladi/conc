# Conc (WIP)

Conc is a simple process manager.

## Architecture

Core of the application (daemon) is written in C, supporting tools (cli, gui, tui) are written in Rust.

### Languages

`/` - Rust

`/apps/daemon` - C

### Folder structure

    .
    ├── apps
    │   ├── daemon              # Systemd daemon handling core of the process management
    │   ├── cli                 # Command line interface communication with the daemon
    │   ├── gui                 # Graphical interface communicating with daemon (Planned)
    │   └── tui                 # Terminal interface communicating with daemon (Planned)
    ├── crates
    │   ├── daemon-client       # Shared Rust library that provides thin layer for communication with the daemon
    │   └── project-settings    # Shared Rust library for searching and parsing project settings
    └── Cargo.toml
