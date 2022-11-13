use serde::Deserialize;
use serde::Serialize;

use crate::AccommodationAddressAvro;

pub const SCHEMA_NAME_CREATE_ACCOMMODATION: &str = "CreateAccommodationAvroV1";

pub const RAW_SCHEMA_CREATE_ACCOMMODATION_V1: &str =
    include_str!("../resources/accommodation/create_accommodation_v1.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccommodationAvro {
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub address: AccommodationAddressAvro,
}
