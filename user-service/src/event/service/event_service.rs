use common_error::AppError;
use sea_orm::{ActiveModelTrait, Set};
use tracing::instrument;

use crate::{common::context::TransactionalContext, event::model::event};

use super::dto::EventDto;

#[instrument(name = "event.service.save", skip_all)]
pub async fn save(tx_context: &TransactionalContext, event: &EventDto) -> Result<(), AppError> {
    // Build the entity from dto
    let e = event::ActiveModel {
        key: Set(event.key.clone()),
        payload: Set(event.payload.clone()),
        partition: Set(event.partition),
        topic: Set(event.topic.clone()),
        ..Default::default()
    };

    tracing::warn!("Save event to topic: {:?}", e.topic);

    // Save entity
    let _ = e.insert(tx_context.db_connection()).await?;

    Ok(())
}
