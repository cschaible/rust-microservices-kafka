use std::sync::Arc;

use opentelemetry_propagator_b3::propagator::Propagator;
use rdkafka::producer::FutureProducer;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use tracing::error;

use crate::event_service;

pub async fn poll_and_send(
    job_synchronization_mutex: Arc<Mutex<bool>>,
    connection: Arc<DatabaseConnection>,
    producer: Arc<FutureProducer>,
    tracing_propagator: Arc<Propagator>,
) {
    let lock = job_synchronization_mutex.try_lock();
    if lock.is_ok() {
        let mut more_events_to_send = true;
        while more_events_to_send {
            more_events_to_send = find_send_delete(
                connection.clone(),
                producer.clone(),
                tracing_propagator.clone(),
            )
            .await;
        }
    }
}

async fn find_send_delete(
    connection: Arc<DatabaseConnection>,
    producer: Arc<FutureProducer>,
    tracing_propagator: Arc<Propagator>,
) -> bool {
    // TODO consider db-transactional
    let event_list = event_service::find_next_page(connection.as_ref()).await;

    match event_list {
        Ok(events) => {
            // Skip further processing if there are no events to send
            if events.events.is_empty() {
                return false;
            }

            // Send data
            event_service::send_to_kafka(producer.clone(), tracing_propagator.clone(), &events)
                .await;

            // Delete sent events
            event_service::delete_from_db(connection.as_ref(), &events)
                .await
                .expect("Delete of sent events from database failed");

            // Send signal to continue without waiting
            events.has_more
        }
        Err(e) => {
            error!("Error occurred while sending events to kafka: {:?}", e);
            false
        }
    }
}
