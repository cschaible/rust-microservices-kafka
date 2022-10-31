use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use common_error::AppError;
use futures::future;
use opentelemetry_propagator_b3::propagator::Propagator;
use opentelemetry_propagator_b3::propagator::B3_SINGLE_HEADER;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::FutureRecord;
use rdkafka::producer::Producer;
use rdkafka::util::Timeout;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseTransaction;
use sea_orm::DeleteResult;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use tracing::info;
use tracing::instrument;
use tracing::span;
use tracing::Instrument;
use tracing::Level;
use tracing_common::B3SpanExt;

use super::super::model::event;
use super::super::model::event::Entity as EventEntity;
use crate::common::db::MAX_PAGE_SIZE;

pub async fn find_next_page(db_connection: &DatabaseTransaction) -> Result<EventList> {
    let page_size = MAX_PAGE_SIZE + 1;

    let events: Vec<event::Model> = EventEntity::find()
        .order_by_asc(event::Column::Id)
        .paginate(db_connection, page_size)
        .fetch_page(0)
        .await?;

    let event_list = EventList {
        has_more: events.len() > MAX_PAGE_SIZE,
        events: events.into_iter().take(MAX_PAGE_SIZE).collect(),
    };

    Ok(event_list)
}

#[instrument(name = "kafka_connector.delete_events", skip_all, level = "trace")]
pub async fn delete_from_db(
    db_connection: &DatabaseTransaction,
    events: &EventList,
) -> Result<u64> {
    let event_ids: Vec<i32> = events
        .events
        .iter()
        .take(MAX_PAGE_SIZE)
        .map(|e| e.id)
        .collect();

    delete(db_connection, event_ids).await
}

async fn delete(db_connection: &DatabaseTransaction, event_ids: Vec<i32>) -> Result<u64> {
    let result: DeleteResult = EventEntity::delete_many()
        .filter(event::Column::Id.is_in(event_ids))
        .exec(db_connection)
        .await?;

    Ok(result.rows_affected)
}

#[instrument(name = "kafka_connector.send_events", skip_all, level = "trace")]
pub async fn send_to_kafka(
    producer: Arc<FutureProducer>,
    tracing_propagator: Arc<Propagator>,
    events: &EventList,
) -> Result<(), AppError> {
    let events_to_send = &events.events;
    let number_of_events = events_to_send.len();

    if number_of_events > 0 {
        let mut event_ids: Vec<i32> = Vec::new();

        info!("Sending {} events", number_of_events);

        // Start kafka transaction
        producer.begin_transaction()?;

        // Send each event individually. Send a span for each message to jaeger.
        let send_result = future::try_join_all(events_to_send.iter().map(|event| {
            let trace_id = event.trace_id.clone();

            // Initialize span
            let span = span!(Level::TRACE, "kafka_connector.send");
            let _ = span.enter();

            if let Some(id) = trace_id.clone() {
                span.set_parent_from_b3(tracing_propagator.clone(), id);
            }

            // Create kafka headers with trace_id
            let headers = match trace_id {
                Some(id) => OwnedHeaders::new_with_capacity(1).add(B3_SINGLE_HEADER, &id),
                _ => OwnedHeaders::default(),
            };

            // Send message to kafka
            // Instrumented<OwnedDeliveryResult>
            let delivery_result = {
                producer.send(
                    FutureRecord::to(&*event.topic)
                        .payload(&event.payload)
                        .partition(event.partition)
                        .key(&event.key)
                        .headers(headers),
                    Duration::from_secs(0),
                )
            }
            .instrument(span);

            // delivery_result.into_inner().expect("Couldn't send event");
            event_ids.push(event.id);

            delivery_result
        }))
        .await;

        match send_result {
            Ok(_) => (),
            Err(e) => return Err(e.0.into()),
        }

        // Commit kafka transaction
        producer.commit_transaction(Timeout::from(Duration::from_secs(30)))?;

        info!("Sent {} events", number_of_events);
    }

    Ok(())
}

pub struct EventList {
    pub has_more: bool,
    pub events: Vec<event::Model>,
}
