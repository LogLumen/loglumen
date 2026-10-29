use axum::{
    Json, Router,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    AppState,
    error::{AppError, AppResult},
    query::{self, Bucket, Query},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list_projects))
        .route("/projects/{project_name}/runs", get(list_runs))
        .route(
            "/projects/{project_name}/runs/{run_name}/metrics",
            get(list_metrics),
        )
        .route(
            "/projects/{project_name}/runs/{run_name}/metrics/{key}/fields",
            get(list_fields),
        )
        .route("/query", post(run_query))
        .route(
            "/projects/{project_name}/dashboards",
            get(list_dashboards).post(create_dashboard),
        )
        .route(
            "/dashboards/{id}",
            get(fetch_dashboard)
                .put(update_dashboard)
                .delete(remove_dashboard),
        )
        .route(
            "/dashboards/{id}/charts",
            get(list_charts).post(create_chart),
        )
        .route(
            "/charts/{id}",
            get(fetch_chart).put(update_chart).delete(remove_chart),
        )
        .route("/charts/ws", get(data_stream))
}

#[derive(Serialize, FromRow)]
struct Project {
    name: String,
}

#[derive(Serialize, FromRow)]
struct Run {
    name: String,
}

#[derive(Serialize, FromRow)]
struct Metric {
    name: String,
}

#[derive(Serialize, FromRow)]
struct Field {
    name: String,
}

#[derive(Serialize, FromRow)]
struct Dashboard {
    id: Uuid,
    project_name: String,
    config: sqlx::types::Json<Value>,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

#[derive(Serialize, FromRow)]
struct Chart {
    id: Uuid,
    dashboard_id: Uuid,
    config: sqlx::types::Json<Value>,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

#[derive(Deserialize)]
struct ConfigBody {
    config: Value,
}

fn dashboard_not_found() -> AppError {
    AppError::not_found("dashboard not found")
}

fn chart_not_found() -> AppError {
    AppError::not_found("chart not found")
}

async fn has_runs_table(state: &AppState) -> AppResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'runs')",
    )
    .fetch_one(&state.client_db.pool)
    .await?)
}

async fn list_projects(State(state): State<AppState>) -> AppResult<Json<Vec<Project>>> {
    if !has_runs_table(&state).await? {
        return Ok(Json(vec![]));
    }
    let projects = sqlx::query_as("SELECT DISTINCT project_name AS name FROM runs ORDER BY name")
        .fetch_all(&state.client_db.pool)
        .await?;
    Ok(Json(projects))
}

async fn list_runs(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
) -> AppResult<Json<Vec<Run>>> {
    if !has_runs_table(&state).await? {
        return Ok(Json(vec![]));
    }
    let runs =
        sqlx::query_as("SELECT run_name AS name FROM runs WHERE project_name = ? ORDER BY name")
            .bind(project_name)
            .fetch_all(&state.client_db.pool)
            .await?;
    Ok(Json(runs))
}

async fn list_metrics(
    State(state): State<AppState>,
    Path((project_name, run_name)): Path<(String, String)>,
) -> AppResult<Json<Vec<Metric>>> {
    if !has_runs_table(&state).await? {
        return Ok(Json(vec![]));
    }
    let metrics = sqlx::query_as(
        "SELECT DISTINCT m.key AS name FROM metrics m JOIN runs r ON r.id = m.run_id
         WHERE r.project_name = ? AND r.run_name = ? ORDER BY name",
    )
    .bind(project_name)
    .bind(run_name)
    .fetch_all(&state.client_db.pool)
    .await?;
    Ok(Json(metrics))
}

async fn list_fields(
    State(state): State<AppState>,
    Path((project_name, run_name, key)): Path<(String, String, String)>,
) -> AppResult<Json<Vec<Field>>> {
    if !has_runs_table(&state).await? {
        return Ok(Json(vec![]));
    }
    let fields = sqlx::query_as(
        "SELECT DISTINCT m.field AS name FROM metrics m JOIN runs r ON r.id = m.run_id
         WHERE r.project_name = ? AND r.run_name = ? AND m.key = ? ORDER BY name",
    )
    .bind(project_name)
    .bind(run_name)
    .bind(key)
    .fetch_all(&state.client_db.pool)
    .await?;
    Ok(Json(fields))
}

async fn run_query(
    State(state): State<AppState>,
    Json(query): Json<Query>,
) -> AppResult<Json<Vec<Bucket>>> {
    if !has_runs_table(&state).await? {
        return Ok(Json(vec![]));
    }
    Ok(Json(query::run(&state.client_db.pool, &query).await?))
}

async fn list_dashboards(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
) -> AppResult<Json<Vec<Dashboard>>> {
    let dashboards = sqlx::query_as("SELECT * FROM dashboards WHERE project_name = ? ORDER BY id")
        .bind(project_name)
        .fetch_all(&state.db.pool)
        .await?;
    Ok(Json(dashboards))
}

async fn create_dashboard(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
    Json(body): Json<ConfigBody>,
) -> AppResult<(StatusCode, Json<Dashboard>)> {
    let dashboard = sqlx::query_as(
        "INSERT INTO dashboards (id, project_name, config, updated_at) VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(project_name)
    .bind(sqlx::types::Json(body.config))
    .bind(OffsetDateTime::now_utc())
    .fetch_one(&state.db.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(dashboard)))
}

async fn fetch_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Dashboard>> {
    sqlx::query_as("SELECT * FROM dashboards WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .map(Json)
        .ok_or_else(dashboard_not_found)
}

async fn update_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ConfigBody>,
) -> AppResult<Json<Dashboard>> {
    sqlx::query_as("UPDATE dashboards SET config = ?, updated_at = ? WHERE id = ? RETURNING *")
        .bind(sqlx::types::Json(body.config))
        .bind(OffsetDateTime::now_utc())
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .map(Json)
        .ok_or_else(dashboard_not_found)
}

async fn remove_dashboard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM dashboards WHERE id = ?")
        .bind(id)
        .execute(&state.db.pool)
        .await?;
    match result.rows_affected() {
        0 => Err(dashboard_not_found()),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}

async fn list_charts(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
) -> AppResult<Json<Vec<Chart>>> {
    let charts = sqlx::query_as("SELECT * FROM charts WHERE dashboard_id = ? ORDER BY id")
        .bind(dashboard_id)
        .fetch_all(&state.db.pool)
        .await?;
    Ok(Json(charts))
}

async fn create_chart(
    State(state): State<AppState>,
    Path(dashboard_id): Path<Uuid>,
    Json(body): Json<ConfigBody>,
) -> AppResult<(StatusCode, Json<Chart>)> {
    sqlx::query_as(
        "INSERT INTO charts (id, dashboard_id, config, updated_at) SELECT ?, id, ?, ? FROM dashboards WHERE id = ? RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(sqlx::types::Json(body.config))
    .bind(OffsetDateTime::now_utc())
    .bind(dashboard_id)
    .fetch_optional(&state.db.pool)
    .await?
    .map(|chart| (StatusCode::CREATED, Json(chart)))
    .ok_or_else(dashboard_not_found)
}

async fn fetch_chart(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Chart>> {
    sqlx::query_as("SELECT * FROM charts WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .map(Json)
        .ok_or_else(chart_not_found)
}

async fn update_chart(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ConfigBody>,
) -> AppResult<Json<Chart>> {
    sqlx::query_as("UPDATE charts SET config = ?, updated_at = ? WHERE id = ? RETURNING *")
        .bind(sqlx::types::Json(body.config))
        .bind(OffsetDateTime::now_utc())
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await?
        .map(Json)
        .ok_or_else(chart_not_found)
}

async fn remove_chart(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM charts WHERE id = ?")
        .bind(id)
        .execute(&state.db.pool)
        .await?;
    match result.rows_affected() {
        0 => Err(chart_not_found()),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum IncomingMessage {
    Subscribe { id: String, query: Query },
}

#[derive(Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum OutgoingMessage {
    ReplaceAll { id: String, buckets: Vec<Bucket> },
    Error { id: Option<String>, message: String },
}

async fn data_stream(
    State(state): State<AppState>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.recv().await {
        let text = match msg {
            Message::Text(text) => text,
            Message::Close(_) => break,
            _ => continue,
        };
        let reply = handle_text(&state, &text).await.unwrap_or_else(|e| {
            e.log();
            OutgoingMessage::Error {
                id: MessageId::try_from(text.as_str()).ok().map(|m| m.id),
                message: e.client_message().to_owned(),
            }
        });
        let reply = serde_json::to_string(&reply).expect("outgoing messages serialize");
        if socket.send(Message::Text(reply.into())).await.is_err() {
            break;
        }
    }
}

#[derive(Deserialize)]
struct MessageId {
    id: String,
}

impl TryFrom<&str> for MessageId {
    type Error = serde_json::Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(text)
    }
}

async fn handle_text(state: &AppState, text: &str) -> AppResult<OutgoingMessage> {
    let msg = serde_json::from_str(text)?;
    match msg {
        IncomingMessage::Subscribe { id, query } => {
            let buckets = query::run(&state.client_db.pool, &query).await?;
            Ok(OutgoingMessage::ReplaceAll { id, buckets })
        }
    }
}
