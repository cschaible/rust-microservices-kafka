use std::sync::Arc;

use async_trait::async_trait;
use common_error::AppError;
use kafka_common::partition_of;
use kafka_schema_accommodation::schema_create_room_type::CreateRoomTypeAvro;
use kafka_schema_accommodation::schema_create_room_type::SCHEMA_NAME_CREATE_ROOM_TYPE;
use kafka_schema_accommodation::schema_delete_room_type::DeleteRoomTypeAvro;
use kafka_schema_accommodation::schema_delete_room_type::SCHEMA_NAME_DELETE_ROOM_TYPE;
use kafka_schema_accommodation::schema_update_room_type::UpdateRoomTypeAvro;
use kafka_schema_accommodation::schema_update_room_type::SCHEMA_NAME_UPDATE_ROOM_TYPE;
use kafka_schema_accommodation::DATA_TYPE_ROOM_TYPE;
use kafka_schema_common::schema_key::KeyAvro;
use kafka_schema_common::schema_key::SCHEMA_NAME_KEY;
use kafka_schema_common::IdentifierAvro;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::accommodation::model::RoomType;
use crate::event::service::dto::EventDto;
use crate::event::service::dto::SerializableEventDto;
use crate::event::EventConverter;
use crate::TopicConfiguration;

pub struct RoomTypeEventEncoder<'a> {
    pub(crate) avro_encoder: Arc<Mutex<AvroEncoder<'a>>>,
    pub(crate) topic_configuration: TopicConfiguration,
}

impl<'a> RoomTypeEventEncoder<'a> {
    pub fn new(
        avro_encoder: Arc<Mutex<AvroEncoder>>,
        topic_configuration: TopicConfiguration,
    ) -> RoomTypeEventEncoder {
        RoomTypeEventEncoder {
            avro_encoder,
            topic_configuration,
        }
    }
}

#[async_trait]
impl<'a> EventConverter for RoomTypeEventEncoder<'a> {
    fn handles(&self, event_type: String) -> bool {
        matches!(
            event_type.as_str(),
            SCHEMA_NAME_CREATE_ROOM_TYPE
                | SCHEMA_NAME_UPDATE_ROOM_TYPE
                | SCHEMA_NAME_DELETE_ROOM_TYPE
        )
    }

    #[instrument(name = "create_room_type_event_converter.handle", skip_all)]
    async fn handle(
        &mut self,
        event_type: String,
        event: &Box<dyn SerializableEventDto>,
    ) -> Result<EventDto, AppError> {
        let room_type_event = event
            .as_any()
            .downcast_ref::<RoomType>()
            .unwrap_or_else(|| panic!("Unexpected event type detected: {}", event_type));

        // Determine kafka partition
        let partition = partition_of(
            room_type_event.accommodation_id,
            self.topic_configuration.partitions,
        )
        .expect("Invalid partition number detected");

        // Get topic
        let topic = self.topic_configuration.topic.clone();

        let mut avro_encoder = self.avro_encoder.lock().await;

        // Serialize value
        let value_sns = SubjectNameStrategy::RecordNameStrategy(event_type.clone());
        let serialized_value: Vec<u8> = if event_type == *SCHEMA_NAME_CREATE_ROOM_TYPE {
            let create_room_type_avro_avro: CreateRoomTypeAvro = room_type_event.clone().into();
            avro_encoder
                .encode_struct(create_room_type_avro_avro, &value_sns)
                .await?
        } else if event_type == *SCHEMA_NAME_UPDATE_ROOM_TYPE {
            let update_room_type_avro: UpdateRoomTypeAvro = room_type_event.clone().into();
            avro_encoder
                .encode_struct(update_room_type_avro, &value_sns)
                .await?
        } else if event_type == *SCHEMA_NAME_DELETE_ROOM_TYPE {
            let delete_room_type_avro: DeleteRoomTypeAvro = room_type_event.clone().into();
            avro_encoder
                .encode_struct(delete_room_type_avro, &value_sns)
                .await?
        } else {
            panic!("Unhandled event type: {:?}", event_type);
        };

        // Serialize key
        let key_avro = KeyAvro {
            context_identifier: format!("{}", room_type_event.accommodation_id),
            identifier: IdentifierAvro {
                data_type: DATA_TYPE_ROOM_TYPE.to_owned(),
                identifier: format!("{}", room_type_event.id),
                // Use hard codes version -1, to indicate that no optimistic locking is applied
                version: -1,
            },
        };
        let key_sns = SubjectNameStrategy::RecordNameStrategy(SCHEMA_NAME_KEY.to_string());

        let serialized_key = avro_encoder.encode_struct(key_avro, &key_sns).await?;

        // Return dto with required parameters to send it with kafka
        Ok(EventDto {
            topic,
            partition,
            key: serialized_key,
            payload: serialized_value,
        })
    }
}
