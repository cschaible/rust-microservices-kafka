use async_trait::async_trait;
use common_error::AppError;

use self::service::dto::EventDto;
use self::service::dto::SerializableEventDto;

pub mod model;
pub mod service;

pub type DynEventConverter = Box<dyn EventConverter>;

#[async_trait]
pub trait EventConverter: Sync + Send {
    fn handles(&self, event_type: String) -> bool;

    #[allow(clippy::borrowed_box)]
    async fn handle(
        &self,
        event_type: String,
        event: &Box<dyn SerializableEventDto>,
    ) -> Result<EventDto, AppError>;
}

pub fn handles(converter: &DynEventConverter, event_type: String) -> bool {
    converter.handles(event_type)
}

#[derive(Clone)]
pub struct TopicConfiguration {
    pub topic: String,
    pub partitions: i32,
}
