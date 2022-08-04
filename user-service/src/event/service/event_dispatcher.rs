use common_error::AppError;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::event::{handles, DynEventConverter};

use super::dto::{EventDto, SerializableEventDto};

pub struct EventDispatcher {
    pub(crate) event_converters: Vec<Arc<Mutex<DynEventConverter>>>,
}

impl EventDispatcher {
    pub async fn dispatch(
        &mut self,
        event: Box<dyn SerializableEventDto>,
    ) -> Result<Vec<EventDto>, AppError> {
        let mut dtos: Vec<EventDto> = Vec::new();
        let event_type = event.event_type().clone();

        for mutex in self.event_converters.clone().into_iter() {
            let mut converter = mutex.lock().await;
            if handles(&converter, event_type.clone()) {
                dtos.push(converter.handle(&event).await?);
            }
        }

        Ok(dtos)
    }
}
