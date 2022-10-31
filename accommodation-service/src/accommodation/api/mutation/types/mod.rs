use std::sync::Arc;

use common_error::AppError;
use mongodb::ClientSession;

use crate::event::service::dto::SerializableEventDto;
use crate::event::service::event_dispatcher::EventDispatcher;
use crate::event::service::event_service;

pub mod accommodation;
pub mod room_type;

pub async fn create_kafka_events(
    db_session: &ClientSession,
    event_dispatcher: Arc<EventDispatcher>,
    dto: Box<dyn SerializableEventDto>,
    event_type: &str,
) -> Result<(), AppError> {
    let events = event_dispatcher
        .dispatch(event_type.to_string(), dto)
        .await?;

    assert!(!events.is_empty());

    for event in events {
        event_service::save(db_session, &event).await?;
    }
    Ok(())
}
