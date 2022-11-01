use common_error::AppError;
use common_tracing::get_b3_trace_id;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseTransaction;
use sea_orm::Set;
use tracing::instrument;

use super::dto::EventDto;
use crate::event::model::event;

#[instrument(name = "event.service.save", skip_all)]
pub async fn save(db_connection: &DatabaseTransaction, event: &EventDto) -> Result<(), AppError> {
    let trace_id = get_b3_trace_id();

    // Build the entity from dto
    let e = event::ActiveModel {
        key: Set(event.key.clone()),
        payload: Set(event.payload.clone()),
        partition: Set(event.partition),
        topic: Set(event.topic.clone()),
        trace_id: Set(trace_id),
        ..Default::default()
    };

    tracing::debug!("Save event to topic: {:?}", e.topic);

    // Save entity
    let _ = e.insert(db_connection).await?;

    Ok(())
}
