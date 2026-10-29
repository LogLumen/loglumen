use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    routing::{delete, get},
};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", delete(revoke))
}

pub async fn require_api_key(request: Request, next: Next) -> Response {
    // TODO: validate the request's API key
    next.run(request).await
}

async fn list() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

async fn create() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

async fn revoke() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
