use std::sync::Arc;

use async_trait::async_trait;
use common_error::AppError;
use kafka_common::partition_of;
use kafka_schema_common::schema_key::KeyAvro;
use kafka_schema_common::schema_key::SCHEMA_NAME_KEY;
use kafka_schema_common::IdentifierAvro;
use kafka_schema_user::schema_create_user::CreateUserAvro;
use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;
use kafka_schema_user::DATA_TYPE_USER;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use tracing::instrument;

use crate::common::kafka::TopicConfiguration;
use crate::event::service::dto::EventDto;
use crate::event::service::dto::SerializableEventDto;
use crate::event::EventConverter;
use crate::user::event::dto::UserWithPhoneNumbersDto;

pub struct UserEventEncoder<'a> {
    pub(crate) avro_encoder: Arc<AvroEncoder<'a>>,
    pub(crate) topic_configuration: TopicConfiguration,
}

impl<'a> UserEventEncoder<'a> {
    pub fn new(
        avro_encoder: Arc<AvroEncoder>,
        topic_configuration: TopicConfiguration,
    ) -> UserEventEncoder {
        UserEventEncoder {
            avro_encoder,
            topic_configuration,
        }
    }
}

#[async_trait]
impl<'a> EventConverter for UserEventEncoder<'a> {
    fn handles(&self, event_type: String) -> bool {
        matches!(event_type.as_str(), SCHEMA_NAME_CREATE_USER)
    }

    #[instrument(name = "user_event_converter.handle", skip_all)]
    async fn handle(
        &self,
        event_type: String,
        event: &Box<dyn SerializableEventDto>,
    ) -> Result<EventDto, AppError> {
        let user_event = event
            .as_any()
            .downcast_ref::<UserWithPhoneNumbersDto>()
            .unwrap_or_else(|| panic!("Unexpected event type detected: {}", event_type));

        // Determine kafka partition
        let partition = partition_of(
            user_event.user.identifier,
            self.topic_configuration.partitions,
        )
        .expect("Invalid partition number detected");

        // Get topic
        let topic = self.topic_configuration.topic.clone();

        // Serialize value
        let value_sns = SubjectNameStrategy::RecordNameStrategy(event_type.clone());
        let serialized_value: Vec<u8> = if event_type == *SCHEMA_NAME_CREATE_USER {
            let create_user_avro: CreateUserAvro = user_event.clone().into();
            self.avro_encoder
                .encode_struct(create_user_avro, &value_sns)
                .await?
        } else {
            panic!("Unhandled event type: {:?}", event_type);
        };

        // Serialize key
        let key_avro = KeyAvro {
            context_identifier: format!("{}", user_event.user.identifier),
            identifier: IdentifierAvro {
                data_type: DATA_TYPE_USER.to_owned(),
                identifier: format!("{}", user_event.user.identifier),
                version: user_event.user.version,
            },
        };
        let key_sns = SubjectNameStrategy::RecordNameStrategy(SCHEMA_NAME_KEY.to_string());

        let serialized_key = self.avro_encoder.encode_struct(key_avro, &key_sns).await?;

        // Return dto with required parameters to send it with kafka
        Ok(EventDto {
            topic,
            partition,
            key: serialized_key,
            payload: serialized_value,
        })
    }
}
