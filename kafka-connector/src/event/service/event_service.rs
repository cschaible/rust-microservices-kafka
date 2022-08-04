use anyhow::Result;

use sea_orm::{
    ColumnTrait, ConnectionTrait, DeleteResult, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};

use crate::common::db::MAX_PAGE_SIZE;
use tracing::instrument;

use super::super::{model::event, model::event::Entity as EventEntity};

pub async fn poll_and_send<T: ConnectionTrait + Sized>() {}

#[instrument(name = "kafka_connector.find_next_page", skip(connection))]
pub async fn find_next_page<T: ConnectionTrait + Sized>(connection: &T) -> Result<EventList> {
    let page_size = MAX_PAGE_SIZE + 1;

    let events: Vec<event::Model> = EventEntity::find()
        .order_by_asc(event::Column::Id)
        .paginate(connection, page_size)
        .fetch_page(0)
        .await?;

    let event_list = EventList {
        has_more: events.len() > MAX_PAGE_SIZE,
        events: events.into_iter().take(MAX_PAGE_SIZE).collect(),
    };

    Ok(event_list)
}

pub async fn delete<T: ConnectionTrait + Sized>(
    connection: &T,
    event_ids: Vec<i32>,
) -> Result<u64> {
    let result: DeleteResult = EventEntity::delete_many()
        .filter(event::Column::Id.is_in(event_ids))
        .exec(connection)
        .await?;

    Ok(result.rows_affected)
}

pub struct EventList {
    pub has_more: bool,
    pub events: Vec<event::Model>,
}
