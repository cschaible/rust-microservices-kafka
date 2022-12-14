use std::sync::Arc;

use async_trait::async_trait;
use common_error::AppError;
use common_kafka::partition_of;
use kafka_schema_accommodation::schema_create_accommodation::CreateAccommodationAvro;
use kafka_schema_accommodation::schema_create_accommodation::SCHEMA_NAME_CREATE_ACCOMMODATION;
use kafka_schema_accommodation::schema_update_accommodation::UpdateAccommodationAvro;
use kafka_schema_accommodation::schema_update_accommodation::SCHEMA_NAME_UPDATE_ACCOMMODATION;
use kafka_schema_accommodation::DATA_TYPE_ACCOMMODATION;
use kafka_schema_common::schema_key::KeyAvro;
use kafka_schema_common::schema_key::SCHEMA_NAME_KEY;
use kafka_schema_common::IdentifierAvro;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use tracing::instrument;

use crate::accommodation::model::Accommodation;
use crate::common::kafka;
use crate::config::configuration::KafkaConfiguration;
use crate::config::configuration::TopicProperties;
use crate::event::service::dto::EventDto;
use crate::event::service::dto::SerializableEventDto;
use crate::event::EventConverter;

pub struct AccommodationEventEncoder<'a> {
    pub(crate) avro_encoder: Arc<AvroEncoder<'a>>,
    pub(crate) topic_configuration: TopicProperties,
}

impl<'a> AccommodationEventEncoder<'a> {
    pub fn new<'b>(
        config: &'b KafkaConfiguration,
    ) -> Result<AccommodationEventEncoder<'a>, AppError> {
        Ok(AccommodationEventEncoder {
            avro_encoder: Arc::new(kafka::init_avro_encoder(config)?),
            topic_configuration: config.topic.get_mapping("accommodation"),
        })
    }
}

#[async_trait]
impl<'a> EventConverter for AccommodationEventEncoder<'a> {
    fn handles(&self, event_type: String) -> bool {
        matches!(
            event_type.as_str(),
            SCHEMA_NAME_CREATE_ACCOMMODATION | SCHEMA_NAME_UPDATE_ACCOMMODATION
        )
    }

    #[instrument(name = "create_accommodation_event_converter.handle", skip_all)]
    async fn handle(
        &self,
        event_type: String,
        event: &Box<dyn SerializableEventDto>,
    ) -> Result<EventDto, AppError> {
        let accommodation_event = event
            .as_any()
            .downcast_ref::<Accommodation>()
            .unwrap_or_else(|| panic!("Unexpected event type detected: {}", event_type));

        // Determine kafka partition
        let partition = partition_of(accommodation_event.id, self.topic_configuration.partitions)
            .expect("Invalid partition number detected");

        // Serialize value
        let value_sns = SubjectNameStrategy::RecordNameStrategy(event_type.clone());
        let serialized_value: Vec<u8> = if event_type == *SCHEMA_NAME_CREATE_ACCOMMODATION {
            let create_accommodation_avro: CreateAccommodationAvro =
                accommodation_event.clone().into();
            self.avro_encoder
                .encode_struct(create_accommodation_avro, &value_sns)
                .await?
        } else if event_type == *SCHEMA_NAME_UPDATE_ACCOMMODATION {
            let update_accommodation_avro: UpdateAccommodationAvro =
                accommodation_event.clone().into();
            self.avro_encoder
                .encode_struct(update_accommodation_avro, &value_sns)
                .await?
        } else {
            panic!("Unhandled event type: {:?}", event_type);
        };

        // Serialize key
        let key_avro = KeyAvro {
            context_identifier: format!("{}", accommodation_event.id),
            identifier: IdentifierAvro {
                data_type: DATA_TYPE_ACCOMMODATION.to_owned(),
                identifier: format!("{}", accommodation_event.id),
                version: accommodation_event.version,
            },
        };
        let key_sns = SubjectNameStrategy::RecordNameStrategy(SCHEMA_NAME_KEY.to_string());

        let serialized_key = self.avro_encoder.encode_struct(key_avro, &key_sns).await?;

        // Get topic
        let topic = self.topic_configuration.topic_name.clone();

        // Return dto with required parameters to send it with kafka
        Ok(EventDto {
            topic,
            partition,
            key: serialized_key,
            payload: serialized_value,
        })
    }
}
