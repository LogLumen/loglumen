---
name: loglumen-cli
description: Inspect and configure a LogLumen server from the terminal with `loglumen-cli`. Use this skill whenever the user wants to see which projects, runs, metrics or fields were logged, look at logged values (e.g. how loss evolved over steps or differed between runs), or list, add, change, move, resize or remove charts on a LogLumen dashboard, or otherwise work with LogLumen through its API, even if they don't name the CLI.
---

# loglumen-cli

`loglumen-cli` inspects logged metrics and configures LogLumen through its HTTP API. It never touches the database. Run `loglumen-cli <command> --help` for anything not covered here.

## Connection

- Server: set it with `--url` or `LOGLUMEN_URL`. The default is `http://localhost:3000`.
- Admin password: set it with `--admin-password` or `ADMIN_PASSWORD`. A `.env` file in the current directory or a parent is read too.
  - Never print the password or put it on a command line you show the user. Prefer the environment variable or `.env`.
- Both options work before or after any subcommand.
- `unauthorized: pass --admin-password or set ADMIN_PASSWORD` means the password is missing or wrong. Ask the user for it rather than guessing.

## Exploring logged data

Logs are organised as project → run → metric entry key → field. Each logged entry stores several fields under one key, e.g. `train` with `step`, `loss` and `learning_rate`.

| Command | What it does |
|---|---|
| `loglumen-cli projects list` | List the projects that have logged runs |
| `loglumen-cli runs list --project <name>` | List the project's runs |
| `loglumen-cli metrics list --project <name> --run <run>` | List the metric entry keys logged in the run |
| `loglumen-cli fields list --project <name> --run <run> --key <key>` | List the fields logged under the key in the run |

- Output is a JSON array of `{ "name": "..." }`, sorted by name.
- Metrics and fields are listed per run. Runs of one project usually share them, but check another run if one looks incomplete.
- Unknown projects, runs or keys give `[]`, not an error.

## Querying values

```sh
loglumen-cli query --project <name> --key <key> --select <field> --x <field> \
  [--run <run>]... [--filter <json>] [--max-points <n>]
```

- `--select`: the field whose values are aggregated, e.g. `loss`.
- `--x`: the x axis. Use `timestamp` or any other logged field, e.g. `step`.
- `--run`: only include these runs; repeat it for several. Without it, all runs of the project are mixed together.
- `--max-points`: the x range is split into at most this many equal-width buckets. The default is 100.
- Output is the buckets, sorted by `x`:

```json
[{ "x": 4.5, "avg": 0.81, "min": 0.27, "max": 2.35, "count": 10 }]
```

- `x` is the average x of the bucket's points; `count` is how many points fell into it.
- `timestamp` is in milliseconds since the Unix epoch.
- To compare runs, query each run separately with one `--run` each.
- To see individual points, set `--max-points` at or above the number of points.

### Filters

`--filter` takes the API's JSON filter. It is combined with `--run` using `and`.

| Filter | Meaning |
|---|---|
| `{"type": "cmp", "field": <field>, "op": "eq\|ne\|gt\|gte\|lt\|lte", "value": <number>}` | Compare a field with a number |
| `{"type": "in", "field": <field>, "values": [<number>, ...]}` | Field is one of the numbers |
| `{"type": "run_in", "runs": ["<run>", ...]}` | Only these runs |
| `{"type": "run_not_in", "runs": ["<run>", ...]}` | Exclude these runs |
| `{"type": "and" \| "or", "filters": [<filter>, ...]}` | Combine filters |

- `<field>` is `"timestamp"` or `{"metric": "<field name>"}`, e.g. `{"metric": "step"}`. A bare field name such as `"step"` is rejected.
- Lists must not be empty.
- Example, steps 100 to 199 only:

```sh
loglumen-cli query --project p --key train --select loss --x step \
  --filter '{"type":"and","filters":[{"type":"cmp","field":{"metric":"step"},"op":"gte","value":100},{"type":"cmp","field":{"metric":"step"},"op":"lt","value":200}]}'
```

## Charts

| Command | What it does |
|---|---|
| `loglumen-cli charts list (--dashboard <id> \| --project <name>)` | List the dashboard's charts |
| `loglumen-cli charts get <chart-id>` | Show one chart |
| `loglumen-cli charts create (--dashboard <id> \| --project <name>) (--config <json> \| --config-file <path>)` | Add a chart |
| `loglumen-cli charts update <chart-id> (--config <json> \| --config-file <path>)` | Replace a chart's config |
| `loglumen-cli charts delete <chart-id>` | Delete a chart; prints nothing |

- `--project` uses the project's oldest dashboard, which is the one the UI shows.
- Output is the API response as pretty JSON:

```json
{ "id": "<uuid>", "dashboard_id": "<uuid>", "config": { ... }, "updated_at": "<timestamp>" }
```

### Chart config

```json
{
  "title": "Training loss",
  "key": "train",
  "select": "loss",
  "x": "step",
  "layout": { "x": 0, "y": 0, "w": 3, "h": 2 }
}
```

- `key`: the metric entry key the values were logged under. Find it with `metrics list`.
- `select`: the logged field to plot. Find it with `fields list`.
- `x`: the x axis. Use `step`, `timestamp`, or any other logged field.
- `layout`: the chart's position and size, in grid cells.
  - The grid is 6 columns wide, and each row is 200px tall.
  - `x + w` must not exceed 6.
  - Charts must not overlap.

## Workflows

- **Finding what was logged:** `projects list`, then `runs list`, `metrics list` and `fields list`, narrowing down step by step.
- **Inspecting a metric:** `query` it with a modest `--max-points` (10–20) for an overview, then narrow the x range with a filter to zoom in.
- **Adding a chart:** find the `key` and fields first, and `query` them to check they hold data. Then `list` the existing charts, then pick a `layout` that doesn't overlap them. The first free row is the largest `y + h`.
- **Changing a chart:** `update` replaces the whole config. `get` the chart first, edit its `config`, then send the full object back.
- **Moving or resizing a chart:** change only `layout` that way.
- **Large configs:** for anything beyond a one-liner, write the JSON to a file and pass `--config-file`. This avoids shell-quoting problems.

## Errors

- `project <name> has no dashboard`: the project has no dashboard yet. The user must open it in the UI first. `list` prints `[]` in this case.
- `404 Not Found: chart not found` / `dashboard not found`: the id is wrong or was deleted.
- `config is not valid JSON` / `filter is not valid JSON`: the JSON was rejected locally, before any request was sent.
- `422 Unprocessable Entity: ...`: the server rejected the query's shape, e.g. a wrong filter `type` or a bare field name. The message says which part.
- An empty `query` result means no point matched: check the key, field, runs and filter.
