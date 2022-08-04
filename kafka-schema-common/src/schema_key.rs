use serde::{Deserialize, Serialize};

use crate::IdentifierAvro;

pub const SCHEMA_NAME_KEY: &str = "key";

pub const RAW_SCHEMA_KEY: &str = r#"
    {
        "name": "KeyAvro",
        "type": "record",
        "fields": [
            {
                "name": "contextIdentifier",
                "type": "string"
            },
            {
                "name": "identifier",
                "type": {
                    "name": "IdentifierAvro",
                    "type": "record",
                    "fields": [
                        {
                            "name": "identifier",
                            "type": "string"
                        },
                        {
                            "name": "version",
                            "type": "long"
                        },
                        {
                            "name": "dataType",
                            "type": "string"
                        }
                    ]
                }
            }
        ]
    }
"#;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyAvro {
    pub context_identifier: String,
    pub identifier: IdentifierAvro,
}
