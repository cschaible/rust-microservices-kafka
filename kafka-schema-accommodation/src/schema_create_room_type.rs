use serde::Deserialize;
use serde::Serialize;

use crate::BedTypeAvro;

pub const SCHEMA_NAME_CREATE_ROOM_TYPE: &str = "CreateRoomTypeAvroV1";

pub const RAW_SCHEMA_CREATE_ROOM_TPE_V1: &str =
    include_str!("../resources/room_type/create_room_type_v1.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomTypeAvro {
    pub accommodation_id: String,
    pub identifier: String,
    pub size: u16,
    pub balcony: bool,
    pub bed_type: BedTypeAvro,
    pub tv: bool,
    pub wifi: bool,
}
