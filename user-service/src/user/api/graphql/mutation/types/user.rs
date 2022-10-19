use async_graphql::Context;
use async_graphql::InputObject;
use async_graphql::Object;
use common_error::AppError;
use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;
use sea_orm::ActiveValue::Set;
use tracing::instrument;
use uuid::Uuid;

use crate::common::db::transactional2;
use crate::event::service::dto::SerializableEventDto;
use crate::user::api::create_kafka_events;
use crate::user::api::graphql::query::types::user::UserPayload;
use crate::user::api::graphql::shared::types::CountryCode;
use crate::user::event::dto::UserWithPhoneNumbersDto;
use crate::user::model::user;
use crate::user::service::user_service::create_user;
use crate::DynContext;

#[derive(Default)]
pub struct UserInput;

/// An user with all of its properties.
#[Object]
impl UserInput {
    #[instrument(name = "user_input.add_user", skip_all)]
    pub async fn add_user(
        &self,
        ctx: &Context<'_>,
        input: AddUserInput,
    ) -> Result<UserPayload, AppError> {
        let context = ctx.data_unchecked::<DynContext>();
        let saved_user = transactional2(context.clone(), |tx| {
            let user: user::ActiveModel = input.clone().into();
            Box::pin(async move {
                // Save entity to database
                let user = create_user(tx, &user).await?;

                // Create kafka events
                let dto: Box<dyn SerializableEventDto> = Box::new(UserWithPhoneNumbersDto {
                    user: user.clone(),
                    phone_numbers: None,
                });
                create_kafka_events(tx, dto, SCHEMA_NAME_CREATE_USER).await?;

                Ok(user)
            })
        })
        .await?;

        Ok(UserPayload(saved_user))
    }
}

#[derive(Clone, InputObject)]
pub struct AddUserInput {
    name: String,
    email: String,
    country: CountryCode,
}

impl From<AddUserInput> for user::ActiveModel {
    fn from(input: AddUserInput) -> Self {
        user::ActiveModel {
            identifier: Set(Uuid::new_v4()),
            version: Set(0),
            name: Set(input.name),
            email: Set(input.email),
            country: Set(input.country.into()),
            ..Default::default()
        }
    }
}
