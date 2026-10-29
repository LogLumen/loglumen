use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// `message` is shown to the client; errors converted with `?` only expose their status.
#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: Option<String>,
    source: anyhow::Error,
}

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::with_message(StatusCode::NOT_FOUND, message.into())
    }

    fn with_message(status: StatusCode, message: String) -> Self {
        Self {
            status,
            source: anyhow::anyhow!(message.clone()),
            message: Some(message),
        }
    }

    pub fn client_message(&self) -> &str {
        match &self.message {
            Some(message) if !self.status.is_server_error() => message,
            _ => self.status.canonical_reason().unwrap_or("Error"),
        }
    }

    pub fn log(&self) {
        if self.status.is_server_error() {
            eprintln!("Internal error: {:#}", self.source);
        }
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: None,
            source: err.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.log();
        (self.status, Json(json!({ "error": self.client_message() }))).into_response()
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
