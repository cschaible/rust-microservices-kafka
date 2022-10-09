use std::any::Any;

/// Marker trait for convertibility to avro.
pub trait SerializableEventDto: Send + Sync {
    fn event_type(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventDto {
    pub topic: String,
    pub partition: i32,
    pub key: Vec<u8>,
    pub payload: Vec<u8>,
}
