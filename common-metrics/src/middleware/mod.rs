use std::time::Instant;

use axum::extract::MatchedPath;
use axum::http::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Router;

pub trait RouterMetricsExt {
    fn add_metrics_middleware(&self) -> Router;
}

impl RouterMetricsExt for Router {
    fn add_metrics_middleware(&self) -> Self {
        self.clone().route_layer(middleware::from_fn(track_metrics))
    }
}

pub async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let start = Instant::now();

    // Get path
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    // Get HTTP method
    let method = req.method().clone();

    // Call actual function
    let response = next.run(req).await;

    // Calculate duration and response status
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Create labels for metrics
    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    // Update metrics
    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!("http_requests_duration_seconds", latency, &labels);

    // Return actual function response
    response
}
