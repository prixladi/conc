# Conc API

`conca` is an HTTP/JSON API in front of the conc daemon. It forwards requests over the daemon's Unix socket and checks API keys stored in a SQLite database.

## Quick start

```sh
conca keys create my-laptop        # prints the key once, store it
conca serve                        # listens on 127.0.0.1:7777

curl -H "Authorization: Bearer conc_…" localhost:7777/api/v1/projects
```

## API keys

Each key is a random 256-bit token with the `conc_` prefix. The database stores only its SHA-256 hash, so a lost key cannot be recovered; create a new one. Keys do not expire. They stay valid until revoked.

```sh
conca keys create <name>    # name must be unique among active keys
conca keys list [--all]     # --all includes revoked keys
conca keys revoke <name>
```

The database defaults to `~/.conc/api.db` and is created with `0600` permissions. Override the path with `--db` or `CONC_API_DB`.

## Server options

| Flag            | Env                     | Default                    |
| --------------- | ----------------------- | -------------------------- |
| `--bind`        | `CONC_API_BIND`         | `127.0.0.1:7777`           |
| `--socket`      | `CONC_API_SOCKET`       | from `~/.conc/conf.json`   |
| `--cors-origin` | `CONC_API_CORS_ORIGINS` | none (comma separated)     |

Logging is controlled with `RUST_LOG`. The server has no built-in TLS. If you bind to anything other than loopback, put a TLS-terminating reverse proxy in front of it.

Services started through the API run with the daemon's environment plus the project's `env`. The caller's environment is not forwarded.

## Endpoints

Every route under `/api/v1` requires `Authorization: Bearer <key>`. Errors are returned as `{"error": "<code>", "message": "…"}`.

| Method   | Path                                                 | Description                               |
| -------- | ---------------------------------------------------- | ----------------------------------------- |
| `GET`    | `/health`                                            | Server and daemon status (no auth)        |
| `GET`    | `/api/v1/projects`                                   | All projects with their services          |
| `PUT`    | `/api/v1/projects`                                   | Create or replace a project (body: `conc.json`, `cwd` must be absolute) |
| `GET`    | `/api/v1/projects/{project}`                         | One project                               |
| `DELETE` | `/api/v1/projects/{project}`                         | Remove a project                          |
| `GET`    | `/api/v1/projects/{project}/settings`                | Stored project settings                   |
| `POST`   | `/api/v1/projects/{project}/start`                   | Start all services                        |
| `POST`   | `/api/v1/projects/{project}/stop`                    | Stop all services                         |
| `POST`   | `/api/v1/projects/{project}/restart?type=hot\|cold`  | Restart; default from `conf.json`         |
| `POST`   | `/api/v1/projects/{project}/clear-logs`              | Clear logs of all services                |
| `GET`    | `/api/v1/projects/{project}/services/{service}`      | One service                               |
| `POST`   | `…/services/{service}/start`, `/stop`, `/restart?type=…`, `/clear-logs` | Service actions        |
| `GET`    | `…/services/{service}/logs?lines=200`                | Log tail as `text/plain` (max 10000 lines) |

Project response:

```json
{
  "name": "basic-example",
  "running": 1,
  "total": 2,
  "services": [
    { "name": "dir", "status": "running", "pid": 4121, "started_at": 1795519517, "stopped_at": null },
    { "name": "sleeper", "status": "stopped", "pid": null, "started_at": 1795519517, "stopped_at": 1795519520 }
  ]
}
```

`status` is one of `idle`, `running`, `stopped`, `exited`. Timestamps are Unix seconds.

| Error code           | HTTP |
| -------------------- | ---- |
| `unauthorized`       | 401  |
| `bad_request`        | 400  |
| `project_not_found`  | 404  |
| `service_not_found`  | 404  |
| `daemon_unavailable` | 503  |
| `daemon_error`       | 502  |

## Install

`make install` (or `make install_api` from the repository root) installs the binary into `$PREFIX` and creates a systemd user unit, `conca.service`.
