use serde::Deserialize;
use serde::Serialize;

pub mod schema_create_accommodation;
pub mod schema_create_room_type;
pub mod schema_delete_room_type;
pub mod schema_update_accommodation;
pub mod schema_update_room_type;

pub const DATA_TYPE_ACCOMMODATION: &str = "accommodation";

pub const DATA_TYPE_ROOM_TYPE: &str = "roomType";

#[derive(Debug, Deserialize, Serialize)]
pub enum IsoCountryCodeEnumAvro {
    DE,
    US,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccommodationAddressAvro {
    pub street: String,
    pub house_number: u16,
    pub zip_code: String,
    pub city: String,
    pub area: Option<String>,
    pub country: IsoCountryCodeEnumAvro,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BedTypeAvro {
    Single,
    TwinSingle,
    Double,
    King,
}
