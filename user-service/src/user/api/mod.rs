use std::sync::Arc;

use common_error::AppError;
use sea_orm::DatabaseTransaction;

use crate::event::service::dto::SerializableEventDto;
use crate::event::service::event_dispatcher::EventDispatcher;
use crate::event::service::event_service;

pub mod graphql;
pub mod rest;

async fn create_kafka_events(
    db_connection: &DatabaseTransaction,
    event_dispatcher: Arc<EventDispatcher>,
    dto: Box<dyn SerializableEventDto>,
    event_type: &str,
) -> Result<(), AppError> {
    let events = event_dispatcher
        .dispatch(event_type.to_string(), dto)
        .await?;

    assert!(!events.is_empty());

    for event in events {
        event_service::save(db_connection, &event).await?;
    }
    Ok(())
}
