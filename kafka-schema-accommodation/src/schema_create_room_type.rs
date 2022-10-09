use serde::Deserialize;
use serde::Serialize;

use crate::BedTypeAvro;

pub const SCHEMA_NAME_CREATE_ROOM_TYPE: &str = "CreateRoomTypeAvroV1";

pub const RAW_SCHEMA_CREATE_ROOM_TPE_V1: &str = r#"
    {
        "name": "CreateRoomTypeAvroV1",
        "type": "record",
        "fields": [
            {
                "name": "accommodationId",
                "type": "string"
            },
            {
                "name": "identifier",
                "type": "string"
            },
            {
                "name": "size",
                "type": "int"
            },
            {
                "name": "balcony",
                "type": "boolean"
            },
            {
                "name": "bedType",
                "type": {
                            "name": "BedTypeEnumAvro",
                            "symbols": [
                                "SINGLE",
                                "TWIN_SINGLE",
                                "DOUBLE",
                                "KING"
                            ],
                            "type": "enum"
                        }
                
            },
            {
                "name": "tv",
                "type": "boolean"
            },
            {
                "name": "wifi",
                "type": "boolean"
            }
        ]
    }
"#;

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
