use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::common::model::IsoCountryCodeEnum;

#[derive(Clone, Deserialize, Serialize)]
pub struct Accommodation {
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub id: Uuid,
    pub version: i64,
    pub name: String,
    pub description: String,
    pub address: Address,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Address {
    pub street: String,
    pub house_number: u16,
    pub zip_code: String,
    pub city: String,
    pub area: Option<String>,
    pub country: IsoCountryCodeEnum,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RoomType {
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub accommodation_id: Uuid,
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub id: Uuid,
    pub size: u16,
    pub balcony: bool,
    pub bed_type: BedType,
    pub tv: bool,
    pub wifi: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum BedType {
    Single,
    TwinSingle,
    Double,
    King,
}
