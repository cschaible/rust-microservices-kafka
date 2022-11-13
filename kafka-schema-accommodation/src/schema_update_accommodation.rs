use serde::Deserialize;
use serde::Serialize;

use crate::AccommodationAddressAvro;

pub const SCHEMA_NAME_UPDATE_ACCOMMODATION: &str = "UpdateAccommodationAvroV1";

pub const RAW_SCHEMA_UPDATE_ACCOMMODATION_V1: &str =
    include_str!("../resources/accommodation/update_accommodation_v1.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccommodationAvro {
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub address: AccommodationAddressAvro,
}
