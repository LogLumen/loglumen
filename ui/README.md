# loglumen ui

SvelteKit (Svelte 5) single-page app for exploring loglumen run metrics. Built with bun, Tailwind v4 (design tokens from `desings/dashboard` in `src/routes/layout.css`) and LayerChart.

## Develop

```sh
bun install
bun run dev        # http://localhost:5173, /api proxied to $LOGLUMEN_API (default http://localhost:3000)
```

```sh
bun run check      # svelte-check / TypeScript
bun run lint       # prettier + eslint
bun run format
bun run test       # vitest
```

## Build & serve

`bun run build` writes a static SPA to `build/` (`index.html` is the fallback for client routes). The axum BE serves it and owns `/api`:

```rust
use tower_http::services::{ServeDir, ServeFile};

let ui = ServeDir::new("ui/build").fallback(ServeFile::new("ui/build/index.html"));
let app = Router::new().nest("/api", api_routes).fallback_service(ui);
```

## Layout

- `src/lib/api/client.ts` – fetch wrapper for `/api` (to be fed by OpenAPI codegen once the BE exposes a spec)
- `src/lib/api/dashboards.ts` – projects, runs, dashboards and charts; seeds a project's first dashboard
- `src/lib/api/stream.svelte.ts` – shared `/api/charts/ws` connection with reactive status
- `src/lib/types/api.ts` – BE payload shapes
- `src/lib/dashboard/` – grid layout logic (`layout.ts`) and series joining/formatting (`series.ts`)
- `src/lib/components/` – UI components (`DashboardGrid`, `MetricChart`, `RunsPanel`, `AppHeader`, `AddChartCard`)

The dashboard is a 6-column grid of 200px rows. Each chart's `config.layout` (`{x, y, w, h}` in cells) is saved via `PUT /api/charts/{id}` after a drag or resize.

Imports from `src/lib` use the `#lib/*` subpath alias and need the file extension (`#lib/dashboard/layout.ts`).
