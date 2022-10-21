use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use common_error::AppError;
use dotenv::dotenv;
use opentelemetry_propagator_b3::propagator::B3Encoding;
use opentelemetry_propagator_b3::propagator::Propagator;
use tracing::info;
use tracing_common::initialize_logging_and_tracing;

use crate::common::api::health;
use crate::common::db::init_db_pool;
use crate::common::kafka::init_producer;
use crate::common::server::shutdown_signal;
use crate::event::service::event_service;
use crate::schedule::run_scheduled_job;

pub mod common;
pub mod event;
pub mod job;
pub mod schedule;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize from .env file
    dotenv().ok();

    // Initialize logging and tracing
    initialize_logging_and_tracing(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), |e| {
        // e.add_directive("kafka_connector=trace".parse().unwrap_or_default())
        e.add_directive(
            format!("{}::event=trace", env!("CARGO_PKG_NAME").replace('-', "_"))
                .parse()
                .unwrap_or_default(),
        )
    })?;

    // Initialize db connection pool
    let db = Arc::new(init_db_pool().await);

    // Initialize kafka producer
    let producer = Arc::new(init_producer());

    // Initialize tracing propagator
    let propagator = Arc::new(Propagator::with_encoding(B3Encoding::SingleHeader));

    // Run scheduled job to poll events from database and send them to kafka
    run_scheduled_job(db, producer, propagator).await;

    // Create health endpoint routing.
    let app = Router::new()
        .route("/health", get(health))
        .layer(opentelemetry_tracing_layer());

    // Start web server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
