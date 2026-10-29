# LogLumen

Lightweight metrics logging. Log values from your code, then explore them on a dashboard or from the terminal.

> **Beta:** LogLumen is in beta. Backwards compatibility is not guaranteed between versions.

## Logging from code

To add logging to your project, use the [Rust client](clients/rust/README.md) or the [Python client](clients/python/README.md).

## CLI

`loglumen-cli` talks to a LogLumen server. With it you can:

- start and stop a local server, which also serves the dashboard UI
- browse what was logged: projects, runs, metrics and fields
- query logged values
- manage dashboard charts

Run `loglumen-cli --help` to see every command.

### Install

```sh
curl -fsSL https://raw.githubusercontent.com/LogLumen/loglumen/master/install.sh | sh
```

This installs `loglumen-cli` and `loglumen-server` into `~/.local/bin` (`/usr/local/bin` as root). Set `LOGLUMEN_VERSION` to pick a release or `LOGLUMEN_INSTALL_DIR` to change the location.

### Running the server

```sh
export ADMIN_PASSWORD=change-me       # or put it in a .env file
loglumen-cli server start --detached  # add --port <port> to change the default 3000
```

Open <http://localhost:3000> and log in with any username and the admin password.

```sh
loglumen-cli server status   # is it running?
loglumen-cli server logs     # print its log
loglumen-cli server stop     # stop it
```

Without `--detached`, the server runs in the foreground until you stop it with Ctrl+C.

### Coding agents

The CLI ships a skill that teaches coding agents how to use it and the Python and Rust clients:

```sh
loglumen-cli skill install                  # Claude Code, into ~/.claude/skills
loglumen-cli skill install --agent cursor   # also: codex, agents
loglumen-cli skill install --project        # into the current project instead
```

### Setup prompt

Paste this into your coding agent to set everything up:

```text
Set up LogLumen for this project:
1. Install the CLI: curl -fsSL https://raw.githubusercontent.com/LogLumen/loglumen/master/install.sh | sh
2. Install its skill for yourself: loglumen-cli skill install
3. Reload skills
4. Load loglumen skill
5. Inspect current project and integrate one of available loglumen clients (rust/python)
6. Add loglumen logs to places that deserve it
7. Start loglumen server
8. Use cli to inspect logs and initialize dashboard fitting project needs
9. Open localhost:<port> with dashboard
```
