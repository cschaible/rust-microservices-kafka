use axum::{routing::get, Router};

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use dotenv::dotenv;

use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::common::server::shutdown_signal;

use crate::common::{api::health, tracing::init_tracing};

use crate::common::db::init_db_pool;
use crate::event::service::event_service;

pub mod common;
pub mod event;

#[tokio::main]
async fn main() {
    // Initialize from .env file
    dotenv().ok();

    // Initialize logging and tracing
    init_tracing();

    // Initialize db connection pool
    let db = Arc::new(init_db_pool().await);

    let producer: Arc<FutureProducer> = Arc::new(
        ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("transactional.id", "kafka-connector")
            .set("queue.buffering.max.ms", "0") // Do not buffer
            .create()
            .expect("Producer creation failed"),
    );
    producer
        .init_transactions(Timeout::from(Duration::from_secs(30)))
        .expect("Transaction initialization failed");

    let mutex = Arc::new(Mutex::new(false));
    let scheduler = JobScheduler::new().expect("Job scheduler couldn't be instantiated");
    let job = Job::new_repeated_async(Duration::from_secs(1), move |_job_id, _lock| {
        let c_mutex = mutex.clone();
        let connection = db.clone();
        let p = producer.clone();
        Box::pin(async move {
            let lock = c_mutex.try_lock();
            if lock.is_ok() {
                let mut send_events = true;
                while send_events {
                    // Load events to send
                    // TODO transactional
                    let event_list = event_service::find_next_page(connection.as_ref()).await;

                    send_events = match event_list {
                        Ok(events) => {
                            let events_to_send = events.events;
                            let number_of_events = events_to_send.len();

                            if number_of_events > 0 {
                                let mut event_ids: Vec<i32> = Vec::new();

                                p.begin_transaction()
                                    .expect("Kafka transaction creation failed");

                                let mut futures: Vec<(i32, i64)> =
                                    Vec::with_capacity(number_of_events);
                                for event in events_to_send {
                                    let result = p
                                        .send(
                                            FutureRecord::to(&*event.topic)
                                                .payload(&event.payload)
                                                .key(&event.key),
                                            Duration::from_secs(0),
                                        )
                                        .await;

                                    futures.push(result.expect("Couldn't send event"));
                                    event_ids.push(event.id);
                                }
                                p.commit_transaction(Timeout::from(Duration::from_secs(30)))
                                    .expect("Commit of kafka transaction failed");

                                event_service::delete(connection.as_ref(), event_ids)
                                    .await
                                    .expect("Delete of sent events failed");

                                info!("Sent {} events", number_of_events);
                                events.has_more
                            } else {
                                false
                            }
                        }
                        Err(e) => {
                            error!("Error occurred while sending events to kafka: {:?}", e);
                            false
                        }
                    }
                }
            } else {
                info!("Processing still locked. Do nothing.");
            }
        })
    })
    .expect("Couldn't instantiate job");
    scheduler.add(job).expect("Couldn't add scheduled job");

    #[cfg(feature = "signal")]
    scheduler.shutdown_on_ctrl_c();
    scheduler.start().expect("Couldn't start job scheduler");

    // Configure routing.
    let app = Router::new().route("/health", get(health));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("listening on {addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    opentelemetry::global::shutdown_tracer_provider();
}
