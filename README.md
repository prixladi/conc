# Conc

Conc is a lightweight service runner for your desktop. You group long-running commands (dev servers, tunnels, watchers, workers) into **projects**, and a background daemon starts, stops, restarts and logs them. You can control it from a terminal, a TUI, a desktop app, VS Code or over HTTP.

![usage gif](./assets/example.gif)

## Components

| Component                        | Binary  | Language   | Description                                                  |
| -------------------------------- | ------- | ---------- | ------------------------------------------------------------ |
| [Daemon](apps/daemon)            | `concd` | C          | Core process manager, runs as a (systemd) user service       |
| [CLI](apps/cli)                  | `concc` | Rust       | Command line interface with an interactive TUI mode          |
| [GUI](apps/gui)                  | `concg` | Rust       | Desktop application ([iced](https://iced.rs))                |
| [API](apps/api)                  | `conca` | Rust       | HTTP/JSON proxy to the daemon with API key authentication    |
| [VS Code extension](apps/vscode) | -       | TypeScript | Sidebar, dashboard with live logs, status bar                |

The daemon is the only required part. Every other component is a client that talks to it over a Unix socket.

## Quick start

```sh
make install_daemon install_cli     # build and install concd + concc
systemctl --user start concd        # start the daemon

cd examples/basic
concc upsert                        # register the project from ./conc.json
concc start basic-example           # start all of its services
concc ps basic-example              # check their status
concc logs basic-example dir        # follow the logs of one service
concc interactive                   # or manage everything from the TUI
```

## Projects

A project is described by a `conc.json` file and registered with `concc upsert [path]`. Running `upsert` again replaces the stored definition.

```json
{
  "name": "my-app",
  "cwd": ".",
  "env": { "NODE_ENV": "development" },
  "services": [
    { "name": "api", "command": ["npm", "run", "dev"], "pwd": "api", "env": { "PORT": 3000 } },
    { "name": "web", "command": ["npm", "run", "dev"], "pwd": "web" },
    { "name": "db-tunnel", "command": ["ssh", "-N", "-L", "5432:localhost:5432", "db-host"] }
  ]
}
```

| Field               | Required | Description                                                                         |
| ------------------- | -------- | ----------------------------------------------------------------------------------- |
| `name`              | yes      | Project name: alphanumeric characters, `-` and `_`                                  |
| `cwd`               | no       | Project working directory, relative to the `conc.json` file. Defaults to its folder |
| `env`               | no       | Environment variables for all services (string or number values)                    |
| `services`          | yes      | At least one service                                                                |
| `services[].name`   | yes      | Unique service name: alphanumeric characters, `-` and `_`                           |
| `services[].command`| yes      | Program and arguments, executed directly (not through a shell)                     |
| `services[].pwd`    | no       | Service working directory, relative to the project `cwd`                            |
| `services[].env`    | no       | Environment variables for this service                                              |

Each service runs in its own session, so stopping it also stops its child processes. It starts with the daemon's environment. On top of it go the caller's environment (see `use_caller_env`), then the service `env`, then the project `env`. When a variable is set more than once, the first source in that order wins. Each service's output goes to a log file managed by the daemon.

A service is always in one of these states:

| Status    | Meaning                                    |
| --------- | ------------------------------------------ |
| `IDLE`    | Has never been started                     |
| `RUNNING` | Process is alive                           |
| `STOPPED` | Was stopped through conc                   |
| `EXITED`  | Process ended on its own (finished or crashed) |

There are two kinds of restart. A **hot** restart restarts only the services that are currently running. A **cold** restart restarts all of them, so stopped services start too. The default is set in the configuration below.

## Usage

### CLI

```text
concc upsert [path]                 Create or replace a project from conc.json (alias: up)
concc projects                      List project names
concc services <project>            List service names of a project
concc ps [project] [service]        Status of all projects, one project or one service
concc start <project> [service]     Start a project or a service
concc stop <project> [service]      Stop a project or a service
concc restart <project> [service]   Restart using the default restart behavior
concc logs <project> [service] [-r] Follow logs (-r prints the log file paths instead)
concc clear-logs <project> [service]  Truncate logs (alias: cl)
concc settings <project>            Print the stored project settings
concc remove <project>              Stop and remove a project (alias: rm)
concc interactive                   Open the TUI (alias: i)
```

In the interactive mode, press <kbd>Tab</kbd> on any page to see its keybinds. The TUI shows a live log preview, supports search with <kbd>/</kbd>, and lets you switch the restart behavior with <kbd>Ctrl</kbd>+<kbd>R</kbd>.

### GUI

Run `concg`. It covers status, start/stop/restart and logs. Creating and removing projects is still done with the CLI.

### VS Code

The extension adds a Conc view to the activity bar and a dashboard with project health and live, colored logs. It can also upsert a `conc.json` from the explorer. See [apps/vscode](apps/vscode/README.md).

### HTTP API

`conca` exposes the daemon as a REST API so that other machines, scripts or web frontends can use it:

```sh
conca keys create my-laptop         # prints an API key once
conca serve                         # listens on 127.0.0.1:7777
curl -H "Authorization: Bearer conc_…" localhost:7777/api/v1/projects
```

See [apps/api](apps/api/README.md) for all endpoints and options.

## Configuration

The Rust clients read an optional `~/.conc/conf.json`:

```json
{
  "use_caller_env": true,
  "daemon_socket_path": "/home/me/.conc/run/conc.sock",
  "log_view_command": ["less", "-R", "+GF"],
  "default_log_preview_mode": "On",
  "default_service_restart_behavior": "Cold"
}
```

| Key                                | Default                     | Description                                              |
| ---------------------------------- | --------------------------- | -------------------------------------------------------- |
| `use_caller_env`                   | `true`                      | Pass the client's environment to started services        |
| `daemon_socket_path`               | `~/.conc/run/conc.sock`     | Daemon socket                                            |
| `log_view_command`                 | `["less", "-R", "+GF"]`     | Program used to view logs, the log path is appended      |
| `default_log_preview_mode`         | `On`                        | TUI log preview: `On`, `Off` or `Fit`                    |
| `default_service_restart_behavior` | `Cold`                      | `Hot` or `Cold`                                          |

The VS Code extension uses the same file and can override it with its own settings. The API does not forward the caller's environment.

The daemon takes command line flags:

```text
concd [-w|--work-dir <dir>] [-l|--log-level <T|D|I|W|E|C>] [-d|--daemon] [-h|--help]
```

The installed systemd unit runs it with the working directory `~/.conc/run`. Project definitions, service metadata, logs and the socket are all kept there.

## Architecture

```text
 concc ─┐
 concg ─┤                       ┌──────────┐   fork/exec   ┌──────────────┐
 VS Code├── Unix socket ───────▶│  concd   │──────────────▶│ your services│
 conca ─┘   (~/.conc/run/       │ (daemon) │               └──────────────┘
   ▲         conc.sock)         └──────────┘                  │ stdout/stderr
   │ HTTP + API key                   │                       ▼
 remote clients                ~/.conc/run/projects/<project>/<service>/log
```

The daemon handles every request over the socket. A request is a list of arguments separated by `0x11` and terminated by `\0`. The shared [`daemon-client`](crates/daemon-client) crate implements the protocol for Rust, and the VS Code extension has its own TypeScript implementation.

```text
.
├── apps
│   ├── daemon              # concd: process management core (C)
│   ├── cli                 # concc: command line interface and TUI
│   ├── gui                 # concg: desktop application
│   ├── api                 # conca: HTTP API with SQLite-backed API keys
│   └── vscode              # VS Code extension (TypeScript)
├── crates
│   ├── app-config          # Loading of ~/.conc/conf.json
│   ├── daemon-client       # Socket protocol client
│   └── project-settings    # Finding, validating and parsing conc.json
├── examples                # Example projects
├── flake.nix               # Nix packages for the daemon and the CLI
└── Makefile                # Installation entry point
```

## Installation

No prebuilt binaries are distributed, so you need to build conc yourself.

**Requirements**

- [make](https://www.gnu.org/software/make/) and [gcc](https://gcc.gnu.org/) for the daemon
- [cargo](https://github.com/rust-lang/cargo) for the CLI, GUI and API
- [Node.js](https://nodejs.org) and the `code` CLI for the VS Code extension
- [systemd](https://systemd.io) (optional) to run the daemon as a user service

**Make targets**

| Target                | Installs                                                                 |
| --------------------- | ------------------------------------------------------------------------ |
| `make install`        | Daemon, CLI and GUI                                                      |
| `make install_daemon` | `concd` and the `concd.service` systemd user unit                        |
| `make install_cli`    | `concc`                                                                  |
| `make install_gui`    | `concg`                                                                  |
| `make install_api`    | `conca` and the `conca.service` systemd user unit                        |
| `make install_vscode` | Packages `conc.vsix` and installs it with `code --install-extension`     |

Binaries go to `/usr/local/bin` and units to `~/.config/systemd/user`. Change these with the `PREFIX` and `SYSTEMD_PREFIX` environment variables. After installing, start the daemon with `systemctl --user start concd`, and enable it with `systemctl --user enable concd` if it should start on login. Without systemd, run `concd --work-dir ~/.conc/run` any other way you like.

**Nix**

The flake provides `packages.daemon`, `packages.cli` and a development shell (`nix develop`). The GUI is not packaged because of its dynamic linking requirements.

### Platform support

Conc relies on POSIX features such as Unix sockets, process groups and `unistd`.

- **Linux**: fully supported, this is the primary development platform.
- **macOS**: tested on macOS Tahoe 26.2. Everything except the systemd units works.
- **Windows**: not supported and not planned.

## Development

```sh
cd apps/daemon && make run          # build and run a daemon with ./run as its work directory
cd apps/cli && cargo run -- ps      # debug builds use ../daemon/run/conc.sock (relative to the current directory)
cargo test                          # Rust tests
cd apps/daemon/tests && make run    # daemon unit tests (built with ASan/UBSan)
```

The daemon also has `make run_debug_address` and `make run_debug_thread` targets for AddressSanitizer and ThreadSanitizer builds, and `make format` / `make tidy` for clang tooling.

## License

[MIT](LICENSE)
