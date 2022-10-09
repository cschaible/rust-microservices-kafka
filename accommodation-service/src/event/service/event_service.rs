use std::sync::atomic::Ordering;

use common_error::AppError;
use mongodb::options::InsertOneOptions;
use tracing::instrument;
use tracing_common::get_b3_trace_id;

use super::dto::EventDto;
use crate::common::context::TransactionalContext;
use crate::common::db::ID_GENERATOR;
use crate::event::model::event::Model;

#[instrument(name = "event.service.save", skip_all)]
pub async fn save(tx_context: &TransactionalContext, event: &EventDto) -> Result<(), AppError> {
    let trace_id = get_b3_trace_id();

    // TODO: Check https://gist.github.com/pestilence669/7189df6177b742042302

    // Build the entity from dto
    let e = Model {
        id: ID_GENERATOR.fetch_add(1, Ordering::SeqCst),
        key: event.key.clone(),
        payload: event.payload.clone(),
        partition: event.partition,
        topic: event.topic.clone(),
        trace_id,
    };

    tracing::debug!("Save event to topic: {:?}", e.topic);

    // Save entity
    let database = tx_context
        .db_client()
        .default_database()
        .expect("No default db specified");
    let collection = database.collection::<Model>("event");
    collection
        .insert_one(e, InsertOneOptions::default())
        .await?;

    Ok(())
}
