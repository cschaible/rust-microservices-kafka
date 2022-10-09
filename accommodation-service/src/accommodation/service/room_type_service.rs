use std::collections::HashMap;

use bson;
use bson::doc;
use bson::Document;
use common_error::AppError;
use futures::TryStreamExt;
use itertools::Itertools;
use mongodb::options::DeleteOptions;
use mongodb::options::FindOneOptions;
use mongodb::options::FindOptions;
use mongodb::options::InsertOneOptions;
use mongodb::options::UpdateOptions;
use mongodb::Collection;
use tracing::instrument;
use uuid::Uuid;

use crate::accommodation::model::RoomType;
use crate::common::context::TransactionalContext;
use crate::common::db::get_collection;

#[instrument(name = "accommodation_service.add_room_type", skip_all)]
pub async fn add_room_type(
    tx_context: &mut TransactionalContext,
    room_type: RoomType,
) -> Result<(), AppError> {
    get_room_type_collection(tx_context)
        .insert_one(room_type, InsertOneOptions::default())
        .await?;

    Ok(())
}

#[instrument(name = "accommodation_service.update_room_type", skip_all)]
pub async fn update_room_type(
    tx_context: &mut TransactionalContext,
    room_type: RoomType,
) -> Result<(), AppError> {
    let filter = id_filter(room_type.id);
    let update = doc! {
        "$set": bson::to_bson(&room_type)?
    };

    get_room_type_collection(tx_context)
        .update_one(filter, update, UpdateOptions::default())
        .await?;

    Ok(())
}

#[instrument(name = "accommodation_service.find_room_type", skip_all)]
pub async fn find_room_type(
    tx_context: &mut TransactionalContext,
    id: Uuid,
) -> Result<Option<RoomType>, AppError> {
    let filter = id_filter(id);

    let room_type = get_room_type_collection(tx_context)
        .find_one(filter, FindOneOptions::default())
        .await?;

    Ok(room_type)
}

#[instrument(name = "accommodation_service.find_room_types", skip_all)]
pub async fn find_room_types(
    tx_context: &mut TransactionalContext,
    accommodation_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, Vec<RoomType>>, AppError> {
    let bson_ids: Vec<bson::Uuid> = accommodation_ids
        .into_iter()
        .map(bson::Uuid::from_uuid_1)
        .collect();

    let filter = doc! {
        "accommodation_id": { "$in" : bson_ids }
    };

    let cursor = get_room_type_collection(tx_context)
        .find(filter, FindOptions::default())
        .await?;

    let room_types: Vec<RoomType> = cursor.try_collect().await?;
    let room_types_by_accommodation_id = room_types
        .into_iter()
        .into_group_map_by(|r| r.accommodation_id);

    Ok(room_types_by_accommodation_id)
}

#[instrument(name = "accommodation_service.delete_room_type", skip_all)]
pub async fn delete_room_type(
    tx_context: &mut TransactionalContext,
    id: Uuid,
) -> Result<u64, AppError> {
    let filter = id_filter(id);

    let delete_result = get_room_type_collection(tx_context)
        .delete_one(filter, DeleteOptions::default())
        .await?;

    Ok(delete_result.deleted_count)
}

fn get_room_type_collection(tx_context: &mut TransactionalContext) -> Collection<RoomType> {
    get_collection::<RoomType>(tx_context, "room_type")
}

fn id_filter(id: Uuid) -> Document {
    doc! {
        "id": as_bson_uuid(id)
    }
}

fn as_bson_uuid(id: Uuid) -> bson::Uuid {
    id.into()
}
