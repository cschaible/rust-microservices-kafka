use async_graphql::dataloader::DataLoader;
use async_graphql::Context;
use async_graphql::Object;
use async_graphql::SimpleObject;
use common_error::AppError;
use uuid::Uuid;

use crate::accommodation::api::query::loaders::RoomTypeLoader;
use crate::accommodation::api::query::types::room_type::RoomTypePayload;
use crate::accommodation::api::shared::types::CountryCode;
use crate::accommodation::model;

pub struct AccommodationPayload(pub model::Accommodation);

/// An accommodation with all of its properties.
#[Object]
impl AccommodationPayload {
    /// Technical identifier of the accommodation
    pub async fn id(&self) -> Uuid {
        self.0.id
    }

    /// The name of the accommodation.
    pub async fn name(&self) -> String {
        self.0.name.clone()
    }

    /// A text description the accommodation.
    async fn description(&self) -> String {
        self.0.description.clone()
    }

    /// List of room types of the accommodation.
    /// Room types are independent resources with their own id.
    pub async fn room_types(&self, ctx: &Context<'_>) -> Result<Vec<RoomTypePayload>, AppError> {
        let room_type_loader = ctx.data_unchecked::<DataLoader<RoomTypeLoader>>();
        let room_types = room_type_loader.load_one(self.0.id).await?;

        Ok(match room_types {
            Some(r) => r.into_iter().map(RoomTypePayload).collect(),
            None => Vec::new(),
        })
    }

    /// The address of the accommodation
    async fn address(&self) -> Address {
        self.0.address.clone().into()
    }
}

/// The address of an accommodation.
#[derive(SimpleObject)]
pub struct Address {
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

impl From<model::Address> for Address {
    fn from(model: model::Address) -> Self {
        Address {
            street: model.street,
            house_number: model.house_number,
            zip_code: model.zip_code,
            city: model.city,
            area: model.area,
            country: model.country.into(),
        }
    }
}
