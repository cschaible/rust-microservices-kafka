use std::collections::HashMap;

use async_graphql::dataloader::*;
use common_db::transaction::transactional;
use common_error::AppError;
use uuid::Uuid;

use crate::user::model::projections::PhoneNumberUserIdentifierProjection;
use crate::user::service::phone_number_service;
use crate::DynContext;

pub struct PhoneNumberLoader {
    context: DynContext,
}

impl PhoneNumberLoader {
    pub fn new(context: DynContext) -> PhoneNumberLoader {
        PhoneNumberLoader { context }
    }
}

#[async_trait::async_trait]
impl Loader<Uuid> for PhoneNumberLoader {
    type Error = AppError;
    type Value = Vec<PhoneNumberUserIdentifierProjection>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let phone_numbers = transactional(self.context.db_connection(), |db_connection| {
            let k = keys.to_vec();
            Box::pin(async move {
                let phone_numbers =
                    phone_number_service::find_all_by_user_identifiers(db_connection, k).await?;
                Ok(phone_numbers)
            })
        })
        .await?;

        Ok(phone_numbers)
    }
}
