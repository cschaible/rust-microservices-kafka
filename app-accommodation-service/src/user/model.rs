use common_security::authentication::UserDetails;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Model {
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub identifier: Uuid,
    pub version: i64,
    pub name: String,
}

impl UserDetails for Model {}
