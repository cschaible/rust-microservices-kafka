use axum::response::IntoResponse;
use serde_json::json;

pub async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}
