use common_db_mongodb::util::get_collection;
use common_error::AppError;
use mongodb::options::InsertOneOptions;
use mongodb::ClientSession;
use mongodb::Collection;
use tracing::instrument;
use uuid::Uuid;

use crate::user::model::Model;

#[instrument(name = "create_user", skip_all)]
pub async fn create_user(
    db_session: &ClientSession,
    identifier: Uuid,
    version: i64,
    name: String,
) -> Result<(), AppError> {
    let user = Model {
        identifier,
        version,
        name,
    };

    let collection: Collection<Model> = get_collection::<Model>(db_session, "user");

    collection
        .insert_one(user, InsertOneOptions::default())
        .await?;

    Ok(())
}
