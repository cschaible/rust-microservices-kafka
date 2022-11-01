use async_graphql::dataloader::DataLoader;
use async_graphql::Context;
use async_graphql::Object;
use common_error::AppError;
use uuid::Uuid;

use crate::user::api::graphql::query::loaders::PhoneNumberLoader;
use crate::user::api::graphql::query::types::phone_number::PhoneNumberPayload;
use crate::user::api::graphql::shared::types::CountryCode;
use crate::user::model::user;

pub struct UserPayload(pub user::Model);

/// A user with all of its properties.
#[Object]
impl UserPayload {
    /// Technical identifier of the user
    pub async fn id(&self) -> Uuid {
        self.0.identifier
    }

    /// Technical version of the user
    pub async fn version(&self) -> i64 {
        self.0.version
    }

    /// The name of the user.
    pub async fn name(&self) -> String {
        self.0.name.clone()
    }

    /// The country
    async fn country(&self) -> CountryCode {
        self.0.country.clone().into()
    }

    /// The email address
    async fn email(&self) -> String {
        self.0.email.clone()
    }

    /// List of phone_numbers.
    pub async fn phone_numbers(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<PhoneNumberPayload>, AppError> {
        let phone_number_loader = ctx.data_unchecked::<DataLoader<PhoneNumberLoader>>();
        let phone_numbers = phone_number_loader.load_one(self.0.identifier).await?;

        Ok(match phone_numbers {
            Some(r) => r.into_iter().map(PhoneNumberPayload).collect(),
            None => Vec::new(),
        })
    }
}
