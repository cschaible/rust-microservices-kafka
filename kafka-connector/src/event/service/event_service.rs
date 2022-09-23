use anyhow::Result;
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use std::sync::Arc;
use std::time::Duration;

use sea_orm::{
    ColumnTrait, ConnectionTrait, DeleteResult, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};

use crate::common::db::MAX_PAGE_SIZE;
use opentelemetry_propagator_b3::propagator::Propagator;
use opentelemetry_propagator_b3::propagator::B3_SINGLE_HEADER;
use tracing::instrument::Instrumented;
use tracing::{info, span, Level};
use tracing::{instrument, Instrument};
use tracing_common::B3SpanExt;

use super::super::{model::event, model::event::Entity as EventEntity};

pub async fn poll_and_send<T: ConnectionTrait + Sized>() {}

#[instrument(
    name = "kafka_connector.find_next_page",
    skip(connection),
    level = "trace"
)]
pub async fn find_next_page<T: ConnectionTrait + Sized>(connection: &T) -> Result<EventList> {
    let page_size = MAX_PAGE_SIZE + 1;

    let events: Vec<event::Model> = EventEntity::find()
        .order_by_asc(event::Column::Id)
        .paginate(connection, page_size)
        .fetch_page(0)
        .await?;

    let event_list = EventList {
        has_more: events.len() > MAX_PAGE_SIZE,
        events: events.into_iter().take(MAX_PAGE_SIZE).collect(),
    };

    Ok(event_list)
}

#[instrument(name = "kafka_connector.delete_events", skip_all, level = "trace")]
pub async fn delete_from_db<T: ConnectionTrait + Sized>(
    connection: &T,
    events: &EventList,
) -> Result<u64> {
    let event_ids: Vec<i32> = (&events.events)
        .iter()
        .take(MAX_PAGE_SIZE)
        .map(|e| e.id)
        .collect();

    delete(connection, event_ids).await
}

async fn delete<T: ConnectionTrait + Sized>(connection: &T, event_ids: Vec<i32>) -> Result<u64> {
    let result: DeleteResult = EventEntity::delete_many()
        .filter(event::Column::Id.is_in(event_ids))
        .exec(connection)
        .await?;

    Ok(result.rows_affected)
}

#[instrument(name = "kafka_connector.send_events", skip_all, level = "trace")]
pub async fn send_to_kafka(
    producer: Arc<FutureProducer>,
    tracing_propagator: Arc<Propagator>,
    events: &EventList,
) {
    let events_to_send = &events.events;
    let number_of_events = events_to_send.len();

    if number_of_events > 0 {
        let mut event_ids: Vec<i32> = Vec::new();

        // Start kafka transaction
        producer
            .begin_transaction()
            .expect("Kafka transaction creation failed");

        // Send each event individually. Send a span for each message to jaeger.
        for event in events_to_send {
            let trace_id = event.trace_id.clone();

            // Initialize span
            let span = span!(Level::TRACE, "kafka_connector.send");
            if let Some(id) = trace_id.clone() {
                span.set_parent_from_b3(tracing_propagator.clone(), id);
            }

            // Create kafka headers with trace_id
            let headers = match trace_id {
                Some(id) => OwnedHeaders::new_with_capacity(1).add(B3_SINGLE_HEADER, &id),
                _ => OwnedHeaders::default(),
            };

            // Send message to kafka
            let delivery_result: Instrumented<OwnedDeliveryResult> = {
                producer
                    .send(
                        FutureRecord::to(&*event.topic)
                            .payload(&event.payload)
                            .partition(event.partition)
                            .key(&event.key)
                            .headers(headers),
                        Duration::from_secs(0),
                    )
                    .await
            }
            .instrument(span);

            delivery_result.into_inner().expect("Couldn't send event");
            event_ids.push(event.id);
        }

        // Commit kafka transaction
        producer
            .commit_transaction(Timeout::from(Duration::from_secs(30)))
            .expect("Commit of kafka transaction failed");

        info!("Sent {} events", number_of_events);
    } /*else {
          debug!("Nothing to send");
      }*/
}

pub struct EventList {
    pub has_more: bool,
    pub events: Vec<event::Model>,
}
