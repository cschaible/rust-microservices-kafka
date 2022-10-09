use serde::Deserialize;
use serde::Serialize;

use crate::AccommodationAddressAvro;

pub const SCHEMA_NAME_CREATE_ACCOMMODATION: &str = "CreateAccommodationAvroV1";

pub const RAW_SCHEMA_CREATE_ACCOMMODATION_V1: &str = r#"
    {
        "name": "CreateAccommodationAvroV1",
        "type": "record",
        "fields": [
            {
                "name": "identifier",
                "type": "string"
            },
            {
                "name": "name",
                "type": "string"
            },
            {
                "name": "description",
                "type": "string"
            },
            {
                "name": "address",
                "type": {
                            "name": "AccommodationAddressAvro",
                            "type": "record",
                            "fields": [
                                {
                                    "name": "street",
                                    "type": "string"
                                },
                                {
                                    "name": "houseNumber",
                                    "type": "int"
                                },
                                {
                                    "name": "zipCode",
                                    "type": "string"
                                },
                                {
                                    "name": "city",
                                    "type": "string"
                                },
                                {
                                    "name": "area",
                                    "type": ["null", "string"],
                                    "default": null
                                },
                                {
                                    "name": "country",
                                    "type": {
                                        "name": "IsoCountryCodeEnumAvro",
                                        "symbols": [
                                            "DE",
                                            "US"
                                        ],
                                        "type": "enum"
                                    }
                                }
                            ]
                        }
                
            }
        ]
    }
"#;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccommodationAvro {
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub address: AccommodationAddressAvro,
}
