use async_trait::async_trait;
use common_error::AppError;

use self::service::dto::{EventDto, SerializableEventDto};

pub mod model;
pub mod service;

pub type DynEventConverter = Box<dyn EventConverter>;

#[async_trait]
pub trait EventConverter: Sync + Send {
    fn event_type(&self) -> String;

    #[allow(clippy::borrowed_box)]
    async fn handle(&mut self, event: &Box<dyn SerializableEventDto>)
        -> Result<EventDto, AppError>;
}

pub fn handles(converter: &DynEventConverter, event_type: String) -> bool {
    converter.event_type() == event_type
}

pub struct TopicConfiguration {
    pub topic: String,
    pub partitions: i32,
}
