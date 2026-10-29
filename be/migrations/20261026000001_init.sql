CREATE TABLE dashboards (
    id BLOB NOT NULL PRIMARY KEY,
    project_name TEXT NOT NULL,
    config TEXT NOT NULL CHECK (json_valid(config)),
    updated_at TEXT NOT NULL
);
CREATE INDEX dashboards_project_name ON dashboards (project_name);

CREATE TABLE charts (
    id BLOB NOT NULL PRIMARY KEY,
    dashboard_id BLOB NOT NULL REFERENCES dashboards (id) ON DELETE CASCADE,
    config TEXT NOT NULL CHECK (json_valid(config)),
    updated_at TEXT NOT NULL
);
CREATE INDEX charts_dashboard_id ON charts (dashboard_id);
