extern crate core;

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
use opentelemetry_propagator_b3::propagator::B3Encoding;
use opentelemetry_propagator_b3::propagator::Propagator;
use tokio::sync::Mutex;
use tower_http::compression::predicate::SizeAbove;
use tower_http::compression::CompressionLayer;
use tracing_common::init_tracing;

use crate::accommodation::event::accommodation_converter::AccommodationEventEncoder;
use crate::accommodation::event::room_type_converter::RoomTypeEventEncoder;
use crate::common::api::health;
use crate::common::context::ContextImpl;
use crate::common::db::init_db_client;
use crate::common::kafka::get_avro_decoder;
use crate::common::kafka::init_consumer;
use crate::common::kafka::AvroRecordDecoder;
use crate::common::server::shutdown_signal;
use crate::event::service::event_dispatcher::EventDispatcher;
use crate::event::DynEventConverter;
use crate::event::TopicConfiguration;
use crate::user::listener::listen;

mod accommodation;
mod common;
mod event;
mod graphql;
mod user;

#[tokio::main]
async fn main() {
    // Initialize from .env file
    dotenv().ok();

    // TODO: https://github.com/mehcode/config-rs

    // Initialize logging and tracing
    init_tracing(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), |e| {
        e // .add_directive("async_graphql::graphql=info".parse().unwrap_or_default())
            .add_directive(
                format!(
                    "{}::user::listener=trace",
                    env!("CARGO_PKG_NAME").replace('-', "_")
                )
                .parse()
                .unwrap_or_default(),
            )
        //.add_directive("rdkafka=trace".parse().unwrap_or_default())
    });

    // Initialize db client
    let db_client = init_db_client()
        .await
        .expect("DB client initialization failed");

    // Initialize schema registry client settings
    let sr_settings = resolve_sr_settings();

    // Construct topic configuration
    let accommodation_topic_configuration = TopicConfiguration {
        topic: "accommodation".to_owned(),
        partitions: 2,
    };

    // Construct avro decoder
    let avro_decoder = Mutex::new(get_avro_decoder(&sr_settings));
    let avro_record_decoder = AvroRecordDecoder { avro_decoder };

    // Construct avro encoder, converter and dispatcher
    let avro_encoder = Arc::new(Mutex::new(get_avro_encoder(&sr_settings)));

    // Accommodation
    let accommodation_encoder = AccommodationEventEncoder::new(
        avro_encoder.clone(),
        accommodation_topic_configuration.clone(),
    );

    let accommodation_event_converter: Arc<Mutex<DynEventConverter>> =
        Arc::new(Mutex::new(Box::new(accommodation_encoder)));

    // Room-type
    let room_type_encoder = RoomTypeEventEncoder::new(
        avro_encoder.clone(),
        accommodation_topic_configuration.clone(),
    );

    let room_type_event_converter: Arc<Mutex<DynEventConverter>> =
        Arc::new(Mutex::new(Box::new(room_type_encoder)));

    let event_dispatcher = EventDispatcher {
        event_converters: vec![accommodation_event_converter, room_type_event_converter],
    };

    // Initialize tracing propagator
    let propagator = Arc::new(Propagator::with_encoding(B3Encoding::SingleHeader));
    let kafka_consumer = init_consumer();

    // Construct request context
    let context = ContextImpl {
        avro_decoder: Arc::new(Mutex::new(avro_record_decoder)),
        client: Arc::new(db_client),
        event_dispatcher: Arc::new(Mutex::new(event_dispatcher)),
    };
    let context: DynContext = Arc::new(context);

    // Construct kafka stream consumer
    let consumer_context = context.clone();
    let consumer_handle = tokio::spawn(async move {
        listen(consumer_context, &kafka_consumer, propagator).await;
    });

    // Configure routing. Configure separate router to not trace /health calls.
    let app = Router::new()
        //.register_user_endpoints()
        .route("/health", get(health))
        .layer(opentelemetry_tracing_layer())
        .layer(Extension(context.clone()))
        .layer(CompressionLayer::new().compress_when(SizeAbove::new(0)));
    //.layer(GlobalConcurrencyLimitLayer::new(10));

    let graphql_router = graphql::routing(context);
    let global_router = app.merge(graphql_router);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3005));
    tracing::info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(global_router.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal(consumer_handle))
        .await
        .unwrap();

    opentelemetry::global::shutdown_tracer_provider();
}
