use async_trait::async_trait;
use common_error::AppError;
use kafka_common::partition_of;
use kafka_schema_common::schema_key::{KeyAvro, SCHEMA_NAME_KEY};
use kafka_schema_common::IdentifierAvro;
use kafka_schema_user::{
    schema_create_user::{CreateUserAvro, SCHEMA_NAME_CREATE_USER},
    PhoneNumberAvro, DATA_TYPE_USER,
};
use kafka_schema_user::{IsoCountryCodeEnumAvro, PhoneNumberTypeEnumAvro};
use schema_registry_converter::{
    async_impl::avro::AvroEncoder, schema_registry_common::SubjectNameStrategy,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::common::model::IsoCountryCodeEnum;
use crate::event::service::dto::SerializableEventDto;
use crate::event::TopicConfiguration;
use crate::event::{service::dto::EventDto, EventConverter};

use super::model::phone_number::PhoneNumberTypeEnum;
use super::service::dto::{PhoneNumberDto, UserWithPhoneNumbersDto};

impl From<PhoneNumberDto> for PhoneNumberAvro {
    fn from(dto: PhoneNumberDto) -> Self {
        PhoneNumberAvro {
            country_code: dto.country_code,
            call_number: dto.call_number,
            phone_number_type: match dto.phone_number_type {
                PhoneNumberTypeEnum::Business => PhoneNumberTypeEnumAvro::Business,
                PhoneNumberTypeEnum::Home => PhoneNumberTypeEnumAvro::Home,
                PhoneNumberTypeEnum::Mobile => PhoneNumberTypeEnumAvro::Mobile,
            },
        }
    }
}

impl From<UserWithPhoneNumbersDto> for CreateUserAvro {
    fn from(dto: UserWithPhoneNumbersDto) -> CreateUserAvro {
        CreateUserAvro {
            identifier: format!("{}", dto.user.identifier),
            name: dto.user.name,
            email: dto.user.email,
            country: match dto.user.country {
                IsoCountryCodeEnum::DE => IsoCountryCodeEnumAvro::DE,
                IsoCountryCodeEnum::US => IsoCountryCodeEnumAvro::US,
            },
            phone_numbers: match dto.phone_numbers {
                Some(phone_numbers) => phone_numbers.into_iter().map(|p| p.into()).collect(),
                None => Vec::with_capacity(0),
            },
        }
    }
}

pub struct UserDtoEventConverter<'a> {
    pub(crate) avro_encoder: Arc<Mutex<AvroEncoder<'a>>>,
    pub(crate) topic_configuration: TopicConfiguration,
}

#[async_trait]
impl<'a> EventConverter for UserDtoEventConverter<'a> {
    fn event_type(&self) -> String {
        SCHEMA_NAME_CREATE_USER.to_string()
    }

    #[instrument(name = "user_event_converter.handle", skip_all)]
    async fn handle(
        &mut self,
        event: &Box<dyn SerializableEventDto>,
    ) -> Result<EventDto, AppError> {
        let user_event = event
            .as_any()
            .downcast_ref::<UserWithPhoneNumbersDto>()
            .unwrap_or_else(|| panic!("Unexpected event type detected: {}", event.event_type()));

        // Determine kafka partition
        let partition = partition_of(
            user_event.user.identifier,
            self.topic_configuration.partitions,
        )
        .expect("Invalid partition number detected");

        // Get topic
        let topic = self.topic_configuration.topic.clone();

        let mut avro_encoder = self.avro_encoder.lock().await;

        // Serialize value
        let create_user_avro: CreateUserAvro = user_event.clone().into();
        let value_sns =
            SubjectNameStrategy::RecordNameStrategy(SCHEMA_NAME_CREATE_USER.to_string());
        let serialized_value = avro_encoder
            .encode_struct(create_user_avro, &value_sns)
            .await?;

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
