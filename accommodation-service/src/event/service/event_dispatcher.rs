use std::sync::Arc;

use common_error::AppError;
use tokio::sync::Mutex;
use tracing::instrument;

use super::dto::EventDto;
use super::dto::SerializableEventDto;
use crate::event::handles;
use crate::event::DynEventConverter;

pub struct EventDispatcher {
    pub(crate) event_converters: Vec<Arc<Mutex<DynEventConverter>>>,
}

impl EventDispatcher {
    #[instrument(name = "event_dispatcher.dispatch", skip_all)]
    pub async fn dispatch(
        &mut self,
        event_type: String,
        event: Box<dyn SerializableEventDto>,
    ) -> Result<Vec<EventDto>, AppError> {
        let mut dtos: Vec<EventDto> = Vec::new();
        let event_type = event.event_type(event_type).clone();

        let mut handled = false;
        for mutex in self.event_converters.clone().into_iter() {
            let mut converter = mutex.lock().await;
            if handles(&converter, event_type.clone()) {
                handled = true;
                dtos.push(converter.handle(event_type.clone(), &event).await?);
            }
        }
        assert!(handled);

        Ok(dtos)
    }
}
