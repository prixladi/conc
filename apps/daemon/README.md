# Conc daemon

Daemon service for conc project.

## Installation

By running ...

```sh
make install
```

... you can install the daemon service as a systemd service running under a current user with root dir in `$HOME/.conc`.

Service is by default stopped. To start it you need to run `sudo systemctl start concd`, if the service does not work you can check its logs using `journalctl -n 10 -fu conc.service`. If you need to change the root directory or add command line arguments (eg. widen log level) you can edit the service file directly it should be located at `/usr/lib/systemd/system/conc.service`.
