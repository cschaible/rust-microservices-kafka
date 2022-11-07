extern crate core;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::Extension;
use axum::routing::get;
use axum::Router;
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use common::context::DynContext;
use common_db_mongodb::pool;
use common_error::AppError;
use common_metrics::middleware::RouterMetricsExt;
use common_security::middleware::RouterSecurityExt;
use opentelemetry_propagator_b3::propagator::B3Encoding;
use opentelemetry_propagator_b3::propagator::Propagator;
use rdkafka::consumer::StreamConsumer;
use tokio::task::JoinHandle;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::compression::predicate::SizeAbove;
use tower_http::compression::CompressionLayer;

use crate::accommodation::event::accommodation_converter::AccommodationEventEncoder;
use crate::accommodation::event::room_type_converter::RoomTypeEventEncoder;
use crate::common::api::health;
use crate::common::context::ContextImpl;
use crate::common::db;
use crate::common::kafka;
use crate::common::kafka::AvroRecordDecoder;
use crate::common::security::OAuthConfiguration;
use crate::common::server::shutdown_signal;
use crate::config::configuration::Configuration;
use crate::config::configuration::KafkaConfiguration;
use crate::config::configuration::ServerConfiguration;
use crate::config::logging_tracing;
use crate::event::service::event_dispatcher::EventDispatcher;
use crate::event::DynEventConverter;
use crate::user::listener::listen;

mod accommodation;
mod common;
mod config;
mod event;
mod graphql;
mod user;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration files
    let config = Configuration::load()?;

    // Initialize logging and tracing
    logging_tracing::init(&config)?;

    // Init db client and create indexes
    let db_client = Arc::new(pool::init_db_client(&config.database).await?);
    db::create_indexes(db_client.clone()).await?;

    // Initialize schema decoder
    let avro_decoder = AvroRecordDecoder::new(&config.kafka)?;

    // Initialize schema encoders
    let accommodation_event_converter: Arc<DynEventConverter> =
        Arc::new(Box::new(AccommodationEventEncoder::new(&config.kafka)?));

    let room_type_event_converter: Arc<DynEventConverter> =
        Arc::new(Box::new(RoomTypeEventEncoder::new(&config.kafka)?));

    // Initialize event dispatcher
    let event_dispatcher = EventDispatcher::new(vec![
        accommodation_event_converter,
        room_type_event_converter,
    ]);

    // Construct request context
    let context = ContextImpl::new_dyn_context(
        Arc::new(avro_decoder),
        db_client,
        Arc::new(event_dispatcher),
    );

    // Initialize kafka consumers
    let mut consumers = kafka::init_consumers(&config.kafka)?;
    let propagator = Arc::new(Propagator::with_encoding(B3Encoding::SingleHeader));
    let user_handle = init_user_kafka_consumer(
        context.clone(),
        &config.kafka,
        &mut consumers,
        propagator.clone(),
    );

    let oauth_configuration = OAuthConfiguration::new(context.clone(), &config.security).await?;

    // Start the web-server
    start_web_server(&config.server, context, oauth_configuration, vec![
        user_handle,
    ])
    .await;

    Ok(())
}

async fn start_web_server(
    config: &ServerConfiguration,
    context: DynContext,
    oauth_configuration: OAuthConfiguration,
    shutdown_handles: Vec<JoinHandle<()>>,
) {
    // Initialize routing
    let routing = init_routing(context, oauth_configuration);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(routing.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal(shutdown_handles))
        .await
        .unwrap();

    // Shutdown tracing provider
    opentelemetry::global::shutdown_tracer_provider();
}

fn init_routing(context: DynContext, oauth_configuration: OAuthConfiguration) -> Router {
    let base_router = Router::new().route("/health", get(health));

    let metrics_router = common_metrics::api::init_routing();

    let graphql_router = graphql::routing(context.clone())
        .add_metrics_middleware()
        .add_auth_middleware()
        .layer(opentelemetry_tracing_layer())
        .layer(ConcurrencyLimitLayer::new(10));

    base_router
        .merge(metrics_router)
        .merge(graphql_router)
        .layer(Extension(context))
        .layer(Extension(oauth_configuration.user_details_service))
        .layer(Extension(oauth_configuration.user_identifier_extractor))
        .layer(Extension(oauth_configuration.token_decoders))
        .layer(Extension(oauth_configuration.token_validator))
        .layer(CompressionLayer::new().compress_when(SizeAbove::new(0)))
}

fn init_user_kafka_consumer(
    context: DynContext,
    config: &KafkaConfiguration,
    kafka_consumers: &mut HashMap<String, StreamConsumer>,
    propagator: Arc<Propagator>,
) -> JoinHandle<()> {
    listen(
        context.clone(),
        config,
        kafka_consumers
            .remove("user")
            .expect("User consumer not initialized"),
        propagator,
    )
}
