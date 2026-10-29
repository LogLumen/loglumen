# loglumen (Python client)

Fire-and-forget metrics logging, built on the [Rust client](../rust/README.md). `log` returns right away and a background thread writes the entries to SQLite.

```python
import loglumen as ll

ll.init()
ll.log("train", step=1, loss=0.42)
```

`init` takes optional `project_name`, `run_name` and `db_path`. The defaults are the same as in the Rust client.
