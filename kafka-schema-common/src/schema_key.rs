use serde::Deserialize;
use serde::Serialize;

use crate::IdentifierAvro;

pub const SCHEMA_NAME_KEY: &str = "KeyAvro";

pub const RAW_SCHEMA_KEY: &str = include_str!("../resources/key.avsc");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyAvro {
    pub context_identifier: String,
    pub identifier: IdentifierAvro,
}
