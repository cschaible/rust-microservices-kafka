use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use common_db_relationaldb::pool;
use common_error::AppError;
use opentelemetry_propagator_b3::propagator::B3Encoding;
use opentelemetry_propagator_b3::propagator::Propagator;

use crate::common::api::health;
use crate::common::kafka::init_producer;
use crate::common::server::shutdown_signal;
use crate::config::configuration::Configuration;
use crate::config::configuration::ServerConfiguration;
use crate::config::logging_tracing;
use crate::event::service::event_service;
use crate::schedule::run_scheduled_job;

pub mod common;
pub mod config;
pub mod event;
pub mod job;
pub mod schedule;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration files
    let config = Configuration::load()?;

    // Initialize logging and tracing
    logging_tracing::init(&config)?;

    // Initialize db connection pool
    let connection_pool = Arc::new(pool::init(&config.database).await?);

    // Initialize kafka producer
    let producer = Arc::new(init_producer(&config.kafka)?);

    // Initialize tracing propagator
    let propagator = Arc::new(Propagator::with_encoding(B3Encoding::SingleHeader));

    // Run scheduled job to poll events from database and send them to kafka
    run_scheduled_job(connection_pool, producer, propagator).await?;

    // Start the web-server
    start_web_server(&config.server).await;

    Ok(())
}

async fn start_web_server(config: &ServerConfiguration) {
    // Initialize routing
    let routing = init_routing();

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(routing.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    // Shutdown tracing provider
    opentelemetry::global::shutdown_tracer_provider();
}

fn init_routing() -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(opentelemetry_tracing_layer())
}
