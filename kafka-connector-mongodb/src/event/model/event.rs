use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct Event {
    pub id: i64,
    pub topic: String,
    pub partition: i32,
    #[serde(with = "serde_bytes")]
    pub key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
    pub trace_id: Option<String>,
}
