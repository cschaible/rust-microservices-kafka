use serde::Deserialize;
use serde::Serialize;

pub const SCHEMA_NAME_DELETE_ROOM_TYPE: &str = "DeleteRoomTypeAvroV1";

pub const RAW_SCHEMA_DELETE_ROOM_TPE_V1: &str =
    include_str!("../resources/room_type/delete_room_type_v1.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRoomTypeAvro {
    pub accommodation_id: String,
    pub identifier: String,
}
