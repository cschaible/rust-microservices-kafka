use async_graphql::Context;
use async_graphql::InputObject;
use async_graphql::Object;
use common_error::AppError;
use common_error::DbError;
use kafka_schema_accommodation::schema_create_accommodation::SCHEMA_NAME_CREATE_ACCOMMODATION;
use kafka_schema_accommodation::schema_update_accommodation::SCHEMA_NAME_UPDATE_ACCOMMODATION;
use query::types::accommodation::AccommodationPayload;
use tracing::instrument;
use uuid::Uuid;

use crate::accommodation::api::query;
use crate::accommodation::api::shared::types::CountryCode;
use crate::accommodation::model::Accommodation;
use crate::accommodation::model::Address;
use crate::accommodation::service::accommodation_service::create_accommodation;
use crate::accommodation::service::accommodation_service::find_accommodation;
use crate::accommodation::service::accommodation_service::update_accommodation;
use crate::common::db::transactional2;
use crate::event::service::dto::SerializableEventDto;
use crate::event::service::event_service;
use crate::DynContext;

#[derive(Default)]
pub struct AccommodationInput;

// Best practices can be found here:
// https://www.apollographql.com/blog/graphql/basics/designing-graphql-mutations/

// Check: https://async-graphql.github.io/async-graphql/en/define_input_object.html

/// An accommodation with all of its properties.
#[Object]
impl AccommodationInput {
    #[instrument(name = "accommodation_input.add_accommodation", skip_all)]
    pub async fn add_accommodation(
        &self,
        ctx: &Context<'_>,
        input: AddAccommodationInput,
    ) -> Result<AccommodationPayload, AppError> {
        let context = ctx.data_unchecked::<DynContext>();
        let saved_accommodation = transactional2(context.clone(), |tx| {
            let accommodation: Accommodation = input.clone().into();
            Box::pin(async move {
                // Save entity to database
                create_accommodation(tx, accommodation.clone()).await?;

                // Create kafka events
                let dto: Box<dyn SerializableEventDto> = Box::new(accommodation.clone());
                let events = tx
                    .dispatch_events(SCHEMA_NAME_CREATE_ACCOMMODATION.to_string(), dto)
                    .await?;
                for event in events {
                    event_service::save(tx, &event).await?;
                }

                Ok(accommodation)
            })
        })
        .await?;

        Ok(AccommodationPayload(saved_accommodation))
    }

    #[instrument(name = "accommodation_input.update_accommodation", skip_all)]
    pub async fn update_accommodation(
        &self,
        ctx: &Context<'_>,
        input: UpdateAccommodationInput,
    ) -> Result<AccommodationPayload, AppError> {
        let context = ctx.data_unchecked::<DynContext>();

        let updated_accommodation = transactional2(context.clone(), |tx| {
            let update = input.clone();
            Box::pin(async move {
                let accommodation = find_accommodation(tx, input.id).await?;
                if let Some(mut accommodation) = accommodation {
                    // Check version
                    if input.version != accommodation.version {
                        return Err(AppError::DbError(DbError::Conflict));
                    }

                    // Save entity to database
                    if let Some(name) = update.name {
                        accommodation.name = name;
                    }
                    if let Some(description) = update.description {
                        accommodation.description = description;
                    }
                    if let Some(address) = update.address {
                        accommodation.address = address.into()
                    }
                    accommodation.version += 1;
                    update_accommodation(tx, accommodation.clone()).await?;

                    // Create kafka events
                    let dto: Box<dyn SerializableEventDto> = Box::new(accommodation.clone());
                    let events = tx
                        .dispatch_events(SCHEMA_NAME_UPDATE_ACCOMMODATION.to_string(), dto)
                        .await?;

                    for event in events {
                        event_service::save(tx, &event).await?;
                    }
                    Ok(accommodation)
                } else {
                    Err(AppError::DbError(DbError::NotFound))
                }
            })
        })
        .await?;

        Ok(AccommodationPayload(updated_accommodation))
    }
}

#[derive(Clone, InputObject)]
pub struct AddAccommodationInput {
    name: String,
    description: String,
    address: AddressInput,
}

impl From<AddAccommodationInput> for Accommodation {
    fn from(input: AddAccommodationInput) -> Self {
        Accommodation {
            id: Uuid::new_v4(),
            version: 0,
            name: input.name,
            description: input.description,
            address: Address {
                street: input.address.street,
                house_number: input.address.house_number,
                zip_code: input.address.zip_code,
                city: input.address.city,
                area: input.address.area,
                country: input.address.country.into(),
            },
        }
    }
}

#[derive(Clone, InputObject)]
pub struct UpdateAccommodationInput {
    id: Uuid,
    version: i64,
    name: Option<String>,
    description: Option<String>,
    address: Option<AddressInput>,
}

/// The address of an accommodation.
#[derive(Clone, InputObject)]
pub struct AddressInput {
    /// The street
    street: String,

    /// House number.
    /// Range: 0 - 15635
    house_number: u16,

    /// Zip code
    zip_code: String,

    /// City
    city: String,

    /// Optional area
    area: Option<String>,

    /// ISO country code
    country: CountryCode,
}

impl From<AddressInput> for Address {
    fn from(input: AddressInput) -> Self {
        Address {
            street: input.street,
            house_number: input.house_number,
            zip_code: input.zip_code,
            city: input.city,
            area: input.area,
            country: input.country.into(),
        }
    }
}
