# loglumen (Rust client)

Fire-and-forget metrics logging. `log!` pushes onto an in-memory queue and returns
right away, and a background task writes the entries to SQLite (WAL mode).

```rust
use loglumen as ll;

ll::Config::default().init().await;
ll::log!("train", step = 1, loss = 42);
```

By default, the project name is the git repo's directory name and the run name is
the UTC start time (`2026-10-25T14-03-12`). Entries from every project go to one shared file, `loglumen/loglumen.sqlite` in the
user data dir (`~/Library/Application Support/` on macOS, `~/.local/share/` on Linux), so a single ui/be instance sees all runs.
Change these with `.project_name(..)`, `.run_name(..)` and `.db_path(..)`.

## Demo

```sh
cargo run --example demo
```

The demo writes to that shared file. Each run adds to it under a new run name.
