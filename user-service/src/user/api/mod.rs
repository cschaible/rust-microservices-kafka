use common_error::AppError;
use crate::common::context::TransactionalContext;
use crate::event::service::dto::SerializableEventDto;
use crate::event::service::event_service;

pub mod graphql;
pub mod rest;

async fn create_kafka_events(
    tx: &mut TransactionalContext,
    dto: Box<dyn SerializableEventDto>,
    event_type: &str,
) -> Result<(), AppError> {
    let events = tx.dispatch_events(event_type.to_string(), dto).await?;

    assert!(!events.is_empty());

    for event in events {
        event_service::save(tx, &event).await?;
    }
    Ok(())
}