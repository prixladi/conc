# Conc for VS Code

A VS Code front end for the conc daemon. It talks to the daemon over the same Unix socket as the **cli** and **gui**.

## Features

- **Projects sidebar** (Conc icon in the activity bar). Shows projects and their services with status icons, uptime and PID. Hover over an item for start/stop/restart/log buttons. Right-click for the rest: hot/cold restart, show settings, clear logs, remove.
- **Dashboard** (`Conc: Open Dashboard`, or click a service). Shows summary tiles, overall health, and one card per project with a status bar per service. It also has a live log viewer with ANSI colors and a Follow toggle.
- **Status bar**: running/total service count and the number of exited services. Click it to open the dashboard.
- **Exit notifications**: you get a notification when a running service exits on its own.
- **Upsert from `conc.json`**: right-click a `conc.json` in the explorer, or run `Conc: Upsert Project from conc.json`.
- **Logs**: open a log in an editor, or follow it in a terminal with the `log_view_command` from `~/.conc/conf.json`.

## Configuration

Settings are read from VS Code first, then from `~/.conc/conf.json` (shared with the cli and gui), then built-in defaults.

| Setting                       | Description                                                          |
| ----------------------------- | -------------------------------------------------------------------- |
| `conc.socketPath`             | Daemon socket. Defaults to `daemon_socket_path` or `~/.conc/run/conc.sock` |
| `conc.defaultRestartBehavior` | `Hot` (running services only) or `Cold` (all services). Defaults to `default_service_restart_behavior` |
| `conc.useCallerEnv`           | Pass VS Code's environment to started services. Defaults to `use_caller_env` |
| `conc.refreshInterval`        | Poll interval in ms (default `1000`)                                 |
| `conc.logLines`               | Number of log lines shown in the dashboard (default `500`)           |
| `conc.notifyOnExit`           | Notify when a service exits (default `true`)                         |

## Build and install

Requires `node`/`npm` and the `code` CLI.

```sh
make install          # npm ci, compile, package conc.vsix, code --install-extension
make package          # only produce conc.vsix
```

You can also run `make install_vscode` from the repository root.

For development, open `apps/vscode` in VS Code, run `npm run watch` and press <kbd>F5</kbd> to start an Extension Development Host.
