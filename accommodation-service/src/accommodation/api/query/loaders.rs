use std::collections::HashMap;

use async_graphql::dataloader::*;
use common_error::AppError;
use uuid::Uuid;

use crate::accommodation::model;
use crate::accommodation::service::room_type_service::find_room_types;
use crate::common::db::transactional2;
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
        let room_types = transactional2(self.context.clone(), |tx| {
            let k = keys.to_vec();
            Box::pin(async move {
                let room_types = find_room_types(tx, k).await?;
                Ok(room_types)
            })
        })
        .await?;

        Ok(room_types)
    }
}