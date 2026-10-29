# Rust client

Add it with `cargo add loglumen`. It runs on tokio. If the project has no tokio runtime yet, add one with `cargo add tokio --features rt-multi-thread,macros`.

## Setup

Call `init` once at startup, inside the tokio runtime, before the first `log!`:

```rust
use loglumen as ll;

#[tokio::main]
async fn main() {
    ll::Config::default().init().await;
}
```

- The builder takes `.project_name(String)`, `.run_name(String)` and `.db_path(PathBuf)`. Leave them unset unless the user asks otherwise.
- Use a multi-threaded runtime. On a current-thread runtime the writer only runs while the code awaits, so a synchronous loop fills the queue and entries are dropped.
- `init` panics if the database file can't be opened.

## Logging

```rust
ll::log!("train", step = step, loss = loss, learning_rate = lr);
```

- The first argument is the metric entry key, a `&'static str` or `String`. Every `name = value` pair is a field.
- Values can be any integer or float type.
- For field names that aren't valid identifiers, or that are only known at runtime, call the function form with an array or `Vec` of `(name, value)` pairs: `ll::log("eval", [("val/loss", loss), ("val/acc", acc)])`. Names can be `&'static str` or `String`. All values in one array must have the same type.

## Behaviour

- `log!` returns right away. A background task writes the entries to SQLite.
- Entries logged before `init` are dropped, with `LogLumen not initialized! Dropped log` on stderr.
- The queue holds 1000 entries. When it is full, new entries are dropped with `Log queue is full` on stderr.
- NaN and infinite values are dropped from the entry, with `Dropped non-finite value of field` on stderr.
