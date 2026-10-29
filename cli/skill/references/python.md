# Python client

Add it with `uv add loglumen` (or `pip install loglumen`). It needs Python 3.13 or newer.

## Setup

Call `init` once at startup, before the first `log`:

```python
import loglumen as ll

ll.init()
```

`init` takes optional `project_name`, `run_name` and `db_path`. Leave them unset unless the user asks otherwise.

## Logging

```python
ll.log("train", step=step, loss=loss, learning_rate=lr)
```

- The first argument is the metric entry key; every keyword argument is a field.
- Values must be `int` or `float`. Anything else raises `TypeError`, so convert tensors and arrays first, e.g. `loss.item()`.
- For field names that aren't valid identifiers, unpack a dict: `ll.log("eval", **{"val/loss": v})`.

## Behaviour

- `log` returns right away. A background thread writes the entries to SQLite.
- Entries logged before `init` are dropped, with `LogLumen not initialized! Dropped log` on stderr.
- The queue holds 1000 entries. When it is full, new entries are dropped with `Log queue is full` on stderr.
- NaN and infinite values are dropped from the entry, with `Dropped non-finite value of field` on stderr.
