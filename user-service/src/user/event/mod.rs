use std::sync::Arc;

use async_trait::async_trait;
use common_error::AppError;
use kafka_common::partition_of;
use kafka_schema_common::schema_key::KeyAvro;
use kafka_schema_common::schema_key::SCHEMA_NAME_KEY;
use kafka_schema_common::IdentifierAvro;
use kafka_schema_user::schema_create_user::CreateUserAvro;
use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;
use kafka_schema_user::IsoCountryCodeEnumAvro;
use kafka_schema_user::PhoneNumberAvro;
use kafka_schema_user::PhoneNumberTypeEnumAvro;
use kafka_schema_user::DATA_TYPE_USER;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::schema_registry_common::SubjectNameStrategy;
use tracing::instrument;

use super::model::phone_number::PhoneNumberTypeEnum;
use crate::common::kafka::TopicConfiguration;
use crate::common::model::IsoCountryCodeEnum;
use crate::event::service::dto::EventDto;
use crate::event::service::dto::SerializableEventDto;
use crate::event::EventConverter;
use crate::user::event::dto::UserWithPhoneNumbersDto;
use crate::user::model::phone_number;

pub mod dto;

impl From<phone_number::ActiveModel> for PhoneNumberAvro {
    fn from(phone_number: phone_number::ActiveModel) -> Self {
        PhoneNumberAvro {
            country_code: phone_number.country_code.clone().unwrap(),
            call_number: phone_number.call_number.clone().unwrap(),
            phone_number_type: match phone_number.phone_number_type.unwrap() {
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
    pub(crate) avro_encoder: Arc<AvroEncoder<'a>>,
    pub(crate) topic_configuration: TopicConfiguration,
}

#[async_trait]
impl<'a> EventConverter for UserDtoEventConverter<'a> {
    fn event_type(&self) -> String {
        SCHEMA_NAME_CREATE_USER.to_string()
    }

    #[instrument(name = "user_event_converter.handle", skip_all)]
    async fn handle(&self, event: &Box<dyn SerializableEventDto>) -> Result<EventDto, AppError> {
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

        // Serialize value
        let create_user_avro: CreateUserAvro = user_event.clone().into();
        let value_sns =
            SubjectNameStrategy::RecordNameStrategy(SCHEMA_NAME_CREATE_USER.to_string());
        let serialized_value = self
            .avro_encoder
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
