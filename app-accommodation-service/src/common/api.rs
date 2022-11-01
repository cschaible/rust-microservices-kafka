use std::sync::atomic::AtomicU16;

use axum::response::IntoResponse;
use serde_json::json;

pub static SERVER_PORT: AtomicU16 = AtomicU16::new(0);

pub async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}
