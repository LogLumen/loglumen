use axum::{Router, http::StatusCode, routing::post};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(ingest))
}

async fn ingest() -> StatusCode {
    // TODO: decode protobuf payload
    StatusCode::NOT_IMPLEMENTED
}
