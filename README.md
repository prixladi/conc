# Conc

Conc is a simple desktop process manager.

## Architecture

The core of the application (daemon) is intended to run as a system service. **Cli** and **gui** run as user applications and communicate with a **daemon** through a Unix socket.

### Languages

`/` - Rust

`/apps/daemon` - C

### Directory structure

    .
    ├── apps
    │   ├── daemon              # Systemd daemon handling the core of the process management
    │   ├── cli                 # Command line interface communication with the daemon
    │   └── gui                 # Graphical interface communicating with daemon
    ├── crates
    │   ├── app-config          # Shared Rust library for app config manipulation
    │   ├── daemon-client       # Shared Rust library that provides a thin layer for communication with the daemon
    │   └── project-settings    # Shared Rust library for searching and parsing project settings
    └── Cargo.toml

## Installation

There is currently no distribution of binaries so you need to compile the project yourself. For the installation process to work, you also need to have `systemd` installed and running. The application does not rely on `systemd` in any way but you will need to run the installed **daemon** in some other way if you are not using `systemd`.

**Requirements**

- [make](https://www.gnu.org/software/make/) - to orchestrate the installation process
- [gcc](https://gcc.gnu.org/) - to compile the **daemon**
- [cargo](https://github.com/rust-lang/cargo) - to compile the **cli** and **gui**

### Linux

Run `make install` to install the **daemon**, the **cli**, and the **gui**, and then run `systemctl --user start concd` to start the **daemon**. This will install `concd` as a systemd user service service and `concc` and `concg` as a binary to the `/usr/local/bin` directory, which can be used from the command line `concc -h`, which prints help info of the **cli** or `concg`, which starts the **gui**.

If you want to install just individual parts, you can run `make install_{daemon/cli/gui}`, note that **daemon** and **cli** are required for conc to work properly, **gui** is optional.

### Other

The project is currently heavily dependent on many **POSIX** features such as `Unix socket` and `unistd` from libc, and it should work on any POSIX compliant system but it was only tested on **Linux**. Because of this it will not work Windows and there is currently not plan to support it.

## Usage

After you have the application installed and the daemon is running, you can can start using the **cli** and **gui**. To see some basic usage check [the examples folder](/examples).

Note that **cli** supports entire functionality of the conc but **gui** does not support inserting and deleting projects, you will still need **cli** for that.
