use common_db_mongodb::util::get_collection;
use common_error::AppError;
use common_tracing::get_b3_trace_id;
use mongodb::options::InsertOneOptions;
use mongodb::ClientSession;
use tracing::instrument;

use super::dto::EventDto;
use crate::event::model::event::Model;

#[instrument(name = "event.service.save", skip_all)]
pub async fn save(db_session: &ClientSession, event: &EventDto) -> Result<(), AppError> {
    let trace_id = get_b3_trace_id();

    // Build the entity from dto
    let e = Model {
        key: event.key.clone(),
        payload: event.payload.clone(),
        partition: event.partition,
        topic: event.topic.clone(),
        trace_id,
    };

    tracing::debug!("Save event to topic: {:?}", e.topic);

    // Save entity
    get_collection(db_session, "event")
        .insert_one(e, InsertOneOptions::default())
        .await?;

    Ok(())
}
