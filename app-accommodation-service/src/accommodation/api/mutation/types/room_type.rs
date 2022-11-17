use async_graphql::Context;
use async_graphql::InputObject;
use async_graphql::Object;
use common_db_mongodb::transaction::transactional;
use common_error::AppError;
use common_error::DbError;
use common_security::authentication::DynAuthenticationHolder;
use futures_util::FutureExt;
use kafka_schema_accommodation::schema_create_room_type::SCHEMA_NAME_CREATE_ROOM_TYPE;
use kafka_schema_accommodation::schema_delete_room_type::SCHEMA_NAME_DELETE_ROOM_TYPE;
use kafka_schema_accommodation::schema_update_room_type::SCHEMA_NAME_UPDATE_ROOM_TYPE;
use uuid::Uuid;

use crate::accommodation::api::mutation::types::create_kafka_events;
use crate::accommodation::api::query::types::room_type::RoomTypePayload;
use crate::accommodation::api::shared::types::BedType;
use crate::accommodation::model::RoomType;
use crate::accommodation::service::room_type_service::add_room_type;
use crate::accommodation::service::room_type_service::delete_room_type;
use crate::accommodation::service::room_type_service::find_room_type;
use crate::accommodation::service::room_type_service::update_room_type;
use crate::DynContext;

/// A type of room including properties.
#[derive(Default)]
pub struct RoomTypeInput;

#[Object]
impl RoomTypeInput {
    pub async fn add_room_type(
        &self,
        ctx: &Context<'_>,
        input: CreateRoomTypeInput,
    ) -> Result<RoomTypePayload, AppError> {
        // Check authentication
        ctx.data_unchecked::<DynAuthenticationHolder>()
            .user_authenticated()?;

        // Get context
        let context = ctx.data_unchecked::<DynContext>();

        // Start transaction and execute query
        let saved_room_type = transactional(context.db_client(), |db_session| {
            let event_dispatcher = context.event_dispatcher();
            let room_type: RoomType = input.clone().into();

            async move {
                // Save entity to database
                add_room_type(db_session, room_type.clone()).await?;

                // Create kafka events
                create_kafka_events(
                    db_session,
                    event_dispatcher,
                    Box::new(room_type.clone()),
                    SCHEMA_NAME_CREATE_ROOM_TYPE,
                )
                .await?;
                Ok(room_type)
            }
            .boxed()
        })
        .await?;

        Ok(RoomTypePayload(saved_room_type))
    }

    pub async fn update_room_type(
        &self,
        ctx: &Context<'_>,
        input: UpdateRoomTypeInput,
    ) -> Result<RoomTypePayload, AppError> {
        // Check authentication
        ctx.data_unchecked::<DynAuthenticationHolder>()
            .user_authenticated()?;

        // Get context
        let context = ctx.data_unchecked::<DynContext>();

        // Start transaction and execute query
        let updated_room_type = transactional(context.db_client(), |db_session| {
            let event_dispatcher = context.event_dispatcher();
            let update = input.clone();

            async move {
                let room_type = find_room_type(db_session, update.id).await?;

                if let Some(mut room_type) = room_type {
                    // Save entity to database
                    if let Some(balcony) = update.balcony {
                        room_type.balcony = balcony;
                    }
                    if let Some(bed_type) = update.bed_type {
                        room_type.bed_type = bed_type.into();
                    }
                    if let Some(size) = update.size {
                        room_type.size = size;
                    }
                    if let Some(tv) = update.tv {
                        room_type.tv = tv;
                    }
                    if let Some(wifi) = update.wifi {
                        room_type.wifi = wifi;
                    }
                    update_room_type(db_session, room_type.clone()).await?;

                    // Create kafka events
                    create_kafka_events(
                        db_session,
                        event_dispatcher,
                        Box::new(room_type.clone()),
                        SCHEMA_NAME_UPDATE_ROOM_TYPE,
                    )
                    .await?;
                    Ok(room_type)
                } else {
                    Err(AppError::DbError(DbError::NotFound))
                }
            }
            .boxed()
        })
        .await?;

        Ok(RoomTypePayload(updated_room_type))
    }

    pub async fn delete_room_type(
        &self,
        ctx: &Context<'_>,
        room_type_id: Uuid,
    ) -> Result<bool, AppError> {
        // Check authentication
        ctx.data_unchecked::<DynAuthenticationHolder>()
            .user_authenticated()?;

        // Get context
        let context = ctx.data_unchecked::<DynContext>();

        // Start transaction and execute query
        let deleted_room = transactional(context.db_client(), |db_session| {
            let event_dispatcher = context.event_dispatcher();

            async move {
                let room_type = find_room_type(db_session, room_type_id).await?;

                if let Some(room_type) = room_type {
                    // Delete from database
                    let delete_result = delete_room_type(db_session, room_type.id).await?;

                    // Create kafka events
                    create_kafka_events(
                        db_session,
                        event_dispatcher,
                        Box::new(room_type.clone()),
                        SCHEMA_NAME_DELETE_ROOM_TYPE,
                    )
                    .await?;
                    Ok(delete_result > 0)
                } else {
                    Ok(false)
                }
            }
            .boxed()
        })
        .await?;

        Ok(deleted_room)
    }
}

#[derive(Clone, InputObject)]
pub struct CreateRoomTypeInput {
    accommodation_id: Uuid,
    size: u16,
    balcony: bool,
    bed_type: BedType,
    tv: bool,
    wifi: bool,
}

impl From<CreateRoomTypeInput> for RoomType {
    fn from(r: CreateRoomTypeInput) -> Self {
        RoomType {
            accommodation_id: r.accommodation_id,
            id: Uuid::new_v4(),
            size: r.size,
            balcony: r.balcony,
            bed_type: r.bed_type.into(),
            tv: r.tv,
            wifi: r.wifi,
        }
    }
}

#[derive(Clone, InputObject)]
pub struct UpdateRoomTypeInput {
    id: Uuid,
    size: Option<u16>,
    balcony: Option<bool>,
    bed_type: Option<BedType>,
    tv: Option<bool>,
    wifi: Option<bool>,
}
