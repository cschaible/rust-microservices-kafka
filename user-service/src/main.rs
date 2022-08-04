use std::{net::SocketAddr, sync::Arc};

use axum::{extract::Extension, routing::get, Router};
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use common::{
    context::DynContext,
    kafka::{get_avro_encoder, resolve_sr_settings},
};
use dotenv::dotenv;

use sea_orm_migration::MigratorTrait;
use tokio::sync::Mutex;
use tower_http::{
    compression::{predicate::SizeAbove, CompressionLayer},
    trace::TraceLayer,
};

use migration::Migrator;

use crate::event::DynEventConverter;
use crate::{
    common::server::shutdown_signal, event::TopicConfiguration, user::event::UserDtoEventConverter,
};
use crate::{
    common::{api::health, tracing::init_tracing},
    event::service::event_dispatcher::EventDispatcher,
};
use crate::{
    common::{context::ContextImpl, db::init_db_pool},
    user::api::routing::UserRouteExt,
};

mod common;
mod event;
mod migration;
mod user;

#[tokio::main]
async fn main() {
    // Initialize from .env file
    dotenv().ok();

    // TODO: https://github.com/mehcode/config-rs

    // Initialize logging and tracing
    init_tracing();

    // Initialize db connection pool
    let db = init_db_pool().await;

    // Migrate database
    match Migrator::up(&db, None).await {
        Ok(_) => tracing::debug!("Finished migration steps"),
        Err(e) => {
            tracing::error!("Failed to apply migration: {:?}", e);
            return;
        }
    }

    // Construct avro encoder
    let sr_settings = resolve_sr_settings();
    let avro_encoder = Mutex::new(get_avro_encoder(&sr_settings));

    // Construct topic configuration
    let user_topic_configuration = TopicConfiguration {
        topic: "user".to_owned(),
        partitions: 6,
    };

    // Construct converter
    let user_encoder = UserDtoEventConverter {
        avro_encoder: Arc::new(avro_encoder),
        topic_configuration: user_topic_configuration,
    };
    let user_event_converter: Arc<Mutex<DynEventConverter>> =
        Arc::new(Mutex::new(Box::new(user_encoder)));

    let event_dispatcher = EventDispatcher {
        event_converters: vec![user_event_converter],
    };

    // Construct request context
    let context = ContextImpl {
        db: Arc::new(db),
        event_dispatcher: Arc::new(Mutex::new(event_dispatcher)),
    };
    let context: DynContext = Arc::new(context);

    // Configure routing. Configure separate router to not trace /health calls.
    let app = Router::new()
        .register_user_endpoints()
        .layer(TraceLayer::new_for_http())
        .route("/health", get(health))
        .layer(opentelemetry_tracing_layer())
        .layer(Extension(context))
        .layer(CompressionLayer::new().compress_when(SizeAbove::new(0)));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    opentelemetry::global::shutdown_tracer_provider();
}
