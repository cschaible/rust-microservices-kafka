use bson;
use bson::doc;
use bson::Document;
use common_error::AppError;
use futures::TryStreamExt;
use mongodb::options::FindOneOptions;
use mongodb::options::FindOptions;
use mongodb::options::InsertOneOptions;
use mongodb::options::UpdateOptions;
use mongodb::Collection;
use tracing::instrument;
use uuid::Uuid;

use crate::accommodation::api::shared::types::CountryCode;
use crate::accommodation::model::Accommodation;
use crate::common::context::TransactionalContext;
use crate::common::db::get_collection;
use crate::common::model::IsoCountryCodeEnum;

#[instrument(name = "accommodation_service.create_accommodation", skip_all)]
pub async fn create_accommodation(
    tx_context: &mut TransactionalContext,
    accommodation: Accommodation,
) -> Result<(), AppError> {
    get_accommodation_collection(tx_context)
        .insert_one(accommodation, InsertOneOptions::default())
        .await?;

    Ok(())
}

#[instrument(name = "accommodation_service.update_accommodation", skip_all)]
pub async fn update_accommodation(
    tx_context: &mut TransactionalContext,
    accommodation: Accommodation,
) -> Result<(), AppError> {
    let filter = id_filter(accommodation.id);
    let update = doc! {
        "$set": bson::to_bson(&accommodation)?
    };

    get_accommodation_collection(tx_context)
        .update_one(filter, update, UpdateOptions::default())
        .await?;

    Ok(())
}

#[instrument(name = "accommodation_service.find_accommodation", skip_all)]
pub async fn find_accommodation(
    tx_context: &mut TransactionalContext,
    id: Uuid,
) -> Result<Option<Accommodation>, AppError> {
    let filter = id_filter(id);

    let accommodation = get_accommodation_collection(tx_context)
        .find_one(filter, FindOneOptions::default())
        .await?;

    Ok(accommodation)
}

#[instrument(name = "accommodation_service.find_accommodations", skip_all)]
pub async fn find_accommodations(
    tx_context: &mut TransactionalContext,
    name: Option<String>,
    country: Option<CountryCode>,
) -> Result<Vec<Accommodation>, AppError> {
    let mut filter = Document::new();
    if let Some(name) = name {
        filter.insert("name", bson::Regex {
            pattern: format!(".*{}.*", name),
            options: "i".to_string(),
        });
    }

    if let Some(country) = country {
        let country_code: IsoCountryCodeEnum = country.into();
        let country_code = bson::to_bson(&country_code)?;
        filter.insert("address.country", country_code);
    }

    let cursor = get_accommodation_collection(tx_context)
        .find(filter, FindOptions::default())
        .await?;

    let accommodations = cursor.try_collect().await?;

    Ok(accommodations)
}

fn get_accommodation_collection(
    tx_context: &mut TransactionalContext,
) -> Collection<Accommodation> {
    get_collection::<Accommodation>(tx_context, "accommodation")
}

fn id_filter(id: Uuid) -> Document {
    doc! {
        "id": as_bson_uuid(id)
    }
}

fn as_bson_uuid(id: Uuid) -> bson::Uuid {
    id.into()
}
