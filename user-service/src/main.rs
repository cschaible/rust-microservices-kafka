use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::Extension;
use axum::routing::get;
use axum::Router;
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use common::context::DynContext;
use common::kafka::get_avro_encoder;
use common::kafka::resolve_sr_settings;
use dotenv::dotenv;
use migration::Migrator;
use sea_orm_migration::MigratorTrait;
use tokio::sync::Mutex;
use tower_http::compression::predicate::SizeAbove;
use tower_http::compression::CompressionLayer;
use tracing_common::init_tracing;

use crate::common::api::health;
use crate::common::context::ContextImpl;
use crate::common::db::init_db_pool;
use crate::common::server::shutdown_signal;
use crate::event::service::event_dispatcher::EventDispatcher;
use crate::event::DynEventConverter;
use crate::event::TopicConfiguration;
use crate::user::api::routing::UserRouteExt;
use crate::user::event::UserDtoEventConverter;

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
    init_tracing(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), |e| {
        e.add_directive(
            "sea_orm::database::transaction=info" // trace
                .parse()
                .unwrap_or_default(),
        )
    });

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
        partitions: 2,
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
        .route("/health", get(health))
        .layer(opentelemetry_tracing_layer())
        .layer(Extension(context))
        .layer(CompressionLayer::new().compress_when(SizeAbove::new(0)));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    opentelemetry::global::shutdown_tracer_provider();
}
