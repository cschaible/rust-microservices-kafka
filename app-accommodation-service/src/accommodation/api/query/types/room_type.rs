use async_graphql::Object;
use common_error::AppError;
use uuid::Uuid;

use crate::accommodation::api::shared::types::BedType;
use crate::accommodation::model;

/// A type of room including properties.
pub struct RoomTypePayload(pub model::RoomType);

#[Object]
impl RoomTypePayload {
    /// Technical identifier of the room type
    async fn id(&self) -> Result<Uuid, AppError> {
        Ok(self.0.id)
    }

    /// Size of the room
    async fn size(&self) -> Result<u16, AppError> {
        Ok(self.0.size)
    }

    /// Room has a balcony
    async fn balcony(&self) -> Result<bool, AppError> {
        Ok(self.0.balcony)
    }

    /// Type of bed in the room
    async fn bed_type(&self) -> Result<BedType, AppError> {
        Ok(self.0.bed_type.clone().into())
    }

    /// Room has a tv
    async fn tv(&self) -> Result<bool, AppError> {
        Ok(self.0.tv)
    }

    /// Room has wifi
    async fn wifi(&self) -> Result<bool, AppError> {
        Ok(self.0.wifi)
    }
}

impl From<model::BedType> for BedType {
    fn from(b: model::BedType) -> Self {
        match b {
            model::BedType::Single => BedType::Single,
            model::BedType::TwinSingle => BedType::TwinSingle,
            model::BedType::Double => BedType::Double,
            model::BedType::King => BedType::King,
        }
    }
}

impl From<BedType> for model::BedType {
    fn from(b: BedType) -> Self {
        match b {
            BedType::Single => model::BedType::Single,
            BedType::TwinSingle => model::BedType::TwinSingle,
            BedType::Double => model::BedType::Double,
            BedType::King => model::BedType::King,
        }
    }
}
