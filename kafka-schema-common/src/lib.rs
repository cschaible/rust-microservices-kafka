use serde::{Deserialize, Serialize};

pub mod schema_key;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifierAvro {
    pub data_type: String,
    pub identifier: String,
    pub version: i64,
}
