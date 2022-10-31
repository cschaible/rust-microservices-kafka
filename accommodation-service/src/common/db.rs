use std::sync::Arc;

use bson::doc;
use common_error::AppError;
use mongodb::options::IndexOptions;
use mongodb::Client;
use mongodb::IndexModel;

use crate::accommodation::model::Accommodation;
use crate::accommodation::model::RoomType;

pub async fn create_indexes(client: Arc<Client>) -> Result<(), AppError> {
    let database = client.default_database().expect("No default db specified");

    // Accommodation.id
    let ix_accommodation_id = IndexModel::builder()
        .keys(doc! {
            "id": 1,
        })
        .options(
            IndexOptions::builder()
                .name(Some("ix_accommodation_id".to_string()))
                .unique(true)
                .build(),
        )
        .build();

    database
        .collection::<Accommodation>("accommodation")
        .create_index(ix_accommodation_id, None)
        .await?;

    // Accommodation.name + Accommodation.address.country
    let ix_accommodation_name_country = IndexModel::builder()
        .keys(doc! {
            "name": 1,
            "address.country": 1
        })
        .options(
            IndexOptions::builder()
                .name(Some("ix_accommodation_name_country".to_string()))
                .unique(false)
                .build(),
        )
        .build();

    database
        .collection::<Accommodation>("accommodation")
        .create_index(ix_accommodation_name_country, None)
        .await?;

    // RoomType.id
    let ix_room_type_id = IndexModel::builder()
        .keys(doc! {
            "id": 1,
        })
        .options(
            IndexOptions::builder()
                .name(Some("ix_room_type_id".to_string()))
                .unique(true)
                .build(),
        )
        .build();

    database
        .collection::<RoomType>("room_type")
        .create_index(ix_room_type_id, None)
        .await?;

    // RoomType.accommodation_id
    let ix_room_type_accommodation_id = IndexModel::builder()
        .keys(doc! {
            "accommodation_id": 1,
        })
        .options(
            IndexOptions::builder()
                .name(Some("ix_room_type_accommodation_id".to_string()))
                .unique(false)
                .build(),
        )
        .build();

    database
        .collection::<RoomType>("room_type")
        .create_index(ix_room_type_accommodation_id, None)
        .await?;

    Ok(())
}
