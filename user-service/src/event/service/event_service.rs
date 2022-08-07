use common_error::AppError;
use sea_orm::{ActiveModelTrait, Set};
use tracing::instrument;
use tracing_common::get_b3_trace_id;

use crate::{common::context::TransactionalContext, event::model::event};

use super::dto::EventDto;

#[instrument(name = "event.service.save", skip_all)]
pub async fn save(tx_context: &TransactionalContext, event: &EventDto) -> Result<(), AppError> {
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
    let _ = e.insert(tx_context.db_connection()).await?;

    Ok(())
}
