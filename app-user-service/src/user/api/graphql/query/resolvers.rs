use async_graphql::Context;
use async_graphql::Object;
use common_db_relationaldb::transaction::transactional;
use common_error::AppError;
use common_security::authentication::DynAuthenticationHolder;
use futures::FutureExt;
use uuid::Uuid;

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
        // Check authentication
        ctx.data_unchecked::<DynAuthenticationHolder>()
            .user_authenticated()?;

        // Get context
        let context = ctx.data_unchecked::<DynContext>();

        // Start transaction and search data
        let users = transactional(context.db_connection(), |db_connection| {
            let user_ids = user_ids.clone();
            async move {
                let users = if let Some(ids) = user_ids {
                    user_service::find_all_by_identifiers(db_connection, ids).await?
                } else {
                    user_service::find_all(db_connection).await?
                };

                Ok(users.into_iter().map(UserPayload).collect())
            }
            .boxed()
        })
        .await?;

        Ok(users)
    }
}
