use std::collections::HashMap;

use async_graphql::dataloader::*;
use common_db_mongodb::transaction::transactional;
use common_error::AppError;
use futures_util::FutureExt;
use uuid::Uuid;

use crate::accommodation::model;
use crate::accommodation::service::room_type_service::find_room_types;
use crate::DynContext;

pub struct RoomTypeLoader {
    context: DynContext,
}

impl RoomTypeLoader {
    pub fn new(context: DynContext) -> RoomTypeLoader {
        RoomTypeLoader { context }
    }
}

#[async_trait::async_trait]
impl Loader<Uuid> for RoomTypeLoader {
    type Error = AppError;
    type Value = Vec<model::RoomType>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let room_types = transactional(self.context.db_client(), |db_session| {
            let k = keys.to_vec();
            async move {
                let room_types = find_room_types(db_session, k).await?;
                Ok(room_types)
            }
            .boxed()
        })
        .await?;

        Ok(room_types)
    }
}
