mod api_keys;
mod charts;
mod db;
mod error;
mod logs;
mod query;

use anyhow::{Result, bail};
use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Basic},
};
use db::Database;
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_database_url")]
    database_url: String,
    admin_password: Option<String>,
    #[serde(default)]
    disable_basic_auth: bool,
    #[serde(default)]
    disable_api_key_check: bool,
    #[serde(default = "loglumen_core::sqlite_db_path")]
    client_db_path: PathBuf,
}

fn default_port() -> u16 {
    3000
}

fn default_database_url() -> String {
    "sqlite://dev.db".to_string()
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub client_db: Arc<Database>,
}

/// Basic auth guard for the UI endpoints: any username, the password must be `ADMIN_PASSWORD`.
async fn require_admin(
    State(admin_password): State<Arc<str>>,
    auth: Option<TypedHeader<Authorization<Basic>>>,
    request: Request,
    next: Next,
) -> Response {
    match auth {
        Some(TypedHeader(Authorization(basic))) if basic.password() == &*admin_password => {
            next.run(request).await
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, r#"Basic realm="loglumen""#)],
        )
            .into_response(),
    }
}

#[derive(RustEmbed)]
#[folder = "../ui/build/"]
struct Ui;

/// Serves the embedded SvelteKit build; unknown client routes fall back to `index.html`.
async fn serve_ui(uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');
    let file = match Ui::get(path) {
        Some(file) => file,
        None if path.starts_with("_app/") => return StatusCode::NOT_FOUND.into_response(),
        None => match Ui::get("index.html") {
            Some(file) => file,
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };

    let hash: String = file
        .metadata
        .sha256_hash()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let etag = format!("\"{hash}\"");
    let cache_control = if path.starts_with("_app/immutable/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    if headers
        .get(header::IF_NONE_MATCH)
        .is_some_and(|value| value.as_bytes() == etag.as_bytes())
    {
        return (
            StatusCode::NOT_MODIFIED,
            [
                (header::CACHE_CONTROL, cache_control.to_owned()),
                (header::ETAG, etag),
            ],
        )
            .into_response();
    }

    (
        [
            (header::CONTENT_TYPE, file.metadata.mimetype().to_owned()),
            (header::CACHE_CONTROL, cache_control.to_owned()),
            (header::ETAG, etag),
        ],
        file.data,
    )
        .into_response()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = envy::from_env::<Config>()?;
    let admin_password: Option<Arc<str>> = if config.disable_basic_auth {
        println!("WARNING: basic auth is disabled, UI endpoints are open");
        None
    } else {
        match config.admin_password {
            Some(password) if !password.is_empty() => Some(password.into()),
            _ => bail!("ADMIN_PASSWORD must be set unless DISABLE_BASIC_AUTH=true"),
        }
    };
    if config.disable_api_key_check {
        println!("WARNING: API key check is disabled, log ingestion is open");
    }
    println!(
        "Reading client logs from {}",
        config.client_db_path.display()
    );

    let db = Database::connect(&config.database_url).await?;
    db.run_migrations().await?;

    let client_db = Database::connect_client(&config.client_db_path).await?;

    let state = AppState {
        db: Arc::new(db),
        client_db: Arc::new(client_db),
    };

    // Log ingestion is guarded by API keys.
    let mut logs = logs::router();
    if !config.disable_api_key_check {
        logs = logs.route_layer(middleware::from_fn(api_keys::require_api_key));
    }

    // The UI and the endpoints it uses sit behind the admin password.
    let mut ui = Router::new()
        .nest("/api-keys", api_keys::router())
        .merge(charts::router());
    let mut spa = Router::new().fallback(serve_ui);
    if let Some(admin_password) = admin_password {
        ui = ui.route_layer(middleware::from_fn_with_state(
            admin_password.clone(),
            require_admin,
        ));
        spa = spa.layer(middleware::from_fn_with_state(
            admin_password,
            require_admin,
        ));
    }

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/logs", logs)
                .merge(ui)
                .fallback(|| async { StatusCode::NOT_FOUND }),
        )
        .with_state(state)
        .fallback_service(spa);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
