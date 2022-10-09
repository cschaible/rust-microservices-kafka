use serde::Deserialize;
use serde::Serialize;

pub const SCHEMA_NAME_DELETE_ROOM_TYPE: &str = "DeleteRoomTypeAvroV1";

pub const RAW_SCHEMA_DELETE_ROOM_TPE_V1: &str = r#"
    {
        "name": "DeleteRoomTypeAvroV1",
        "type": "record",
        "fields": [
            {
                "name": "accommodationId",
                "type": "string"
            },
            {
                "name": "identifier",
                "type": "string"
            }
        ]
    }
"#;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRoomTypeAvro {
    pub accommodation_id: String,
    pub identifier: String,
}
