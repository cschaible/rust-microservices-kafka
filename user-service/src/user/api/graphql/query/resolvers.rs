use async_graphql::Context;
use async_graphql::Object;
use common_error::AppError;
use uuid::Uuid;

use crate::common::db::transactional2;
use crate::user::api::graphql::query::types::user::UserPayload;
use crate::user::service::user_service;
use crate::DynContext;

#[derive(Default)]
pub struct UserResolver;

#[Object]
impl UserResolver {
    /// Get a list of users.
    /// Users can be filtered by identifier.
    pub async fn users<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "optional list of identifiers")] user_ids: Option<Vec<Uuid>>,
    ) -> Result<Vec<UserPayload>, AppError> {
        let context = ctx.data_unchecked::<DynContext>();
        let users = transactional2(context.clone(), |tx| {
            let user_id_filter = user_ids.clone();
            Box::pin(async move {
                let users = if let Some(ids) = user_id_filter {
                    user_service::find_all_by_identifiers(tx.db_connection(), ids).await?
                } else {
                    user_service::find_all(tx.db_connection()).await?
                };

                Ok(users.into_iter().map(UserPayload).collect())
            })
        })
        .await?;

        Ok(users)
    }
}
