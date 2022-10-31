use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::Extension;
use axum::routing::get;
use axum::Router;
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use common::context::DynContext;
use common_db::pool;
use common_error::AppError;
use migration::Migrator;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::compression::predicate::SizeAbove;
use tower_http::compression::CompressionLayer;

use crate::common::api::health;
use crate::common::api::SERVER_PORT;
use crate::common::context::ContextImpl;
use crate::common::db;
use crate::common::kafka;
use crate::common::server::shutdown_signal;
use crate::config::configuration::Configuration;
use crate::config::configuration::ServerConfiguration;
use crate::config::logging_tracing;
use crate::event::service::event_dispatcher::EventDispatcher;
use crate::event::DynEventConverter;
use crate::user::event::user_converter::UserEventEncoder;

pub(crate) mod common;
pub(crate) mod config;
pub(crate) mod event;
pub(crate) mod graphql;
pub(crate) mod migration;
pub(crate) mod user;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration files
    let config = Configuration::load()?;

    // Initialize logging and tracing
    logging_tracing::init(&config)?;

    // Initialize db connection pool and migrate database
    let connection_pool = Arc::new(pool::init(&config.database).await?);
    db::migrate(connection_pool.clone()).await?;

    // Initialize user schema encoder
    let user_event_converter: Arc<DynEventConverter> =
        Arc::new(Box::new(UserEventEncoder::new(&config.kafka)?));

    // Initialize event_dispatcher
    let event_dispatcher = EventDispatcher::new(vec![user_event_converter]);

    // Construct request context
    let context = ContextImpl::new_dyn_context(connection_pool, Arc::new(event_dispatcher));

    // Start the web-server
    start_web_server(&config.server, context).await;

    Ok(())
}

async fn start_web_server(config: &ServerConfiguration, context: DynContext) {
    // Initialize routing
    let routing = init_routing(context);

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

fn init_routing(context: DynContext) -> Router {
    let base_router = Router::new().route("/health", get(health));

    let user_rest_router = user::api::rest::routing::init()
        .layer(opentelemetry_tracing_layer())
        .layer(ConcurrencyLimitLayer::new(10));

    let graphql_router = graphql::routing(context.clone())
        .layer(opentelemetry_tracing_layer())
        .layer(ConcurrencyLimitLayer::new(10));

    base_router
        .merge(user_rest_router)
        .merge(graphql_router)
        .layer(Extension(context))
        .layer(CompressionLayer::new().compress_when(SizeAbove::new(0)))
}
