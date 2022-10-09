use common_error::AppError;
use mongodb::options::InsertOneOptions;
use mongodb::Collection;
use tracing::instrument;
use uuid::Uuid;

use crate::common::context::TransactionalContext;
use crate::common::db::get_collection;
use crate::user::model::Model;

#[instrument(name = "accommodation_service.create_user", skip_all)]
pub async fn create_user(
    tx_context: &mut TransactionalContext,
    identifier: Uuid,
    version: i64,
    name: String,
) -> Result<(), AppError> {
    let user = Model {
        identifier,
        version,
        name,
    };

    let collection: Collection<Model> = get_collection::<Model>(tx_context, "user");

    collection
        .insert_one(user, InsertOneOptions::default())
        .await?;

    Ok(())
}
