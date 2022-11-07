use bson::doc;
use bson::Document;
use common_db_mongodb::util::get_collection;
use common_error::AppError;
use mongodb::options::FindOneOptions;
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

#[instrument(name = "find_user", skip_all)]
pub async fn find_one_by_identifier(
    db_session: &ClientSession,
    identifier: Uuid,
) -> Result<Option<Model>, AppError> {
    let collection: Collection<Model> = get_collection::<Model>(db_session, "user");

    Ok(collection
        .find_one(id_filter(identifier), FindOneOptions::default())
        .await?)
}

fn id_filter(id: Uuid) -> Document {
    doc! {
        "identifier": as_bson_uuid(id)
    }
}

fn as_bson_uuid(id: Uuid) -> bson::Uuid {
    id.into()
}
