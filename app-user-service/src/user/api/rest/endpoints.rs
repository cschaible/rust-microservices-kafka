use axum::extract::Extension;
use axum::extract::Path;
use axum::extract::Query;
use axum::Json;
use common_db_relationaldb::transaction::transactional;
use common_error::AppError;
use common_error::DbError;
use common_security::authentication::DynAuthenticationHolder;
use futures::FutureExt;
use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;
use sea_orm::ActiveValue;
use tracing::instrument;
use uuid::Uuid;

use crate::common::context::DynContext;
use crate::common::paging::Page;
use crate::common::paging::PageParams;
use crate::event::service::dto::SerializableEventDto;
use crate::user::api::create_kafka_events;
use crate::user::api::rest::resources::request::create_user_resource::CreateUserResource;
use crate::user::api::rest::resources::response::build_user_resource;
use crate::user::api::rest::resources::response::build_user_resource_page_from_page;
use crate::user::api::rest::resources::response::build_user_resource_page_from_vec;
use crate::user::api::rest::resources::response::build_user_resources;
use crate::user::api::rest::resources::response::user_resource::UserResource;
use crate::user::event::dto::UserWithPhoneNumbersDto;
use crate::user::model::phone_number;
use crate::user::model::user;
use crate::user::service::phone_number_service;
use crate::user::service::user_service;

#[instrument(name = "user.api.create_user", skip_all)]
pub async fn create_user<'a>(
    Json(create_user_resource): Json<CreateUserResource>,
    Extension(context): Extension<DynContext>,
    Extension(authentication): Extension<DynAuthenticationHolder>,
) -> Result<Json<UserResource>, AppError> {
    let user_identifier = authentication.new_user_authenticated()?.user_identifier;
    transactional(context.db_connection(), |db_connection| {
        let mut user: user::ActiveModel = create_user_resource.clone().into();
        let phone_numbers: Option<Vec<phone_number::ActiveModel>> =
            create_user_resource.clone().into();
        let event_dispatcher = context.event_dispatcher();

        async move {
            user.identifier = ActiveValue::set(user_identifier);
            let user = user_service::create_user(db_connection, &user).await?;

            let phone_numbers = if let Some(numbers) = phone_numbers {
                // Set references to user
                let updated_numbers: Vec<phone_number::ActiveModel> = numbers
                    .into_iter()
                    .map(|mut n| {
                        n.user_id = ActiveValue::set(user.id);
                        n
                    })
                    .collect();

                // Save phone numbers in database
                phone_number_service::save(db_connection, updated_numbers.clone()).await?;

                Some(updated_numbers)
            } else {
                None
            };

            let dto: Box<dyn SerializableEventDto> = Box::new(UserWithPhoneNumbersDto {
                user: user.clone(),
                phone_numbers,
            });

            // Create kafka events
            create_kafka_events(
                db_connection,
                event_dispatcher,
                dto,
                SCHEMA_NAME_CREATE_USER,
            )
            .await?;

            Ok(build_user_resource(db_connection, user).await?.into())
        }
        .boxed()
    })
    .await
}

#[instrument(
    name = "user.api.find_one_by_identifier",
    skip(context, authentication)
)]
pub async fn find_one_by_identifier(
    Path(identifier): Path<Uuid>,
    Extension(context): Extension<DynContext>,
    Extension(authentication): Extension<DynAuthenticationHolder>,
) -> Result<Json<UserResource>, AppError> {
    authentication.user_authenticated()?;
    transactional(context.db_connection(), |db_connection| {
        async move {
            match user_service::find_one_by_identifier(db_connection, identifier).await {
                Ok(found_user) => match found_user {
                    Some(user) => Ok(build_user_resource(db_connection, user).await?.into()),
                    None => Err(AppError::DbError(DbError::NotFound)),
                },
                Err(e) => Err(e.into()),
            }
        }
        .boxed()
    })
    .await
}

#[instrument(name = "user.api.find_all", skip(context, authentication))]
pub async fn find_all(
    Query(page_params): Query<PageParams>,
    Extension(context): Extension<DynContext>,
    Extension(authentication): Extension<DynAuthenticationHolder>,
) -> Result<Json<Page<UserResource>>, AppError> {
    authentication.user_authenticated()?;
    transactional(context.db_connection(), |db_connection| {
        let params = page_params.clone();

        async move {
            if let (Some(_page), Some(_page_size)) = (params.page, params.page_size) {
                match user_service::find_all_paged(db_connection, params).await {
                    Ok(users) => Ok(build_user_resource_page_from_page(db_connection, users)
                        .await?
                        .into()),
                    Err(e) => Err(e.into()),
                }
            } else {
                match user_service::find_all(db_connection).await {
                    Ok(users) => {
                        let resource_page =
                            build_user_resource_page_from_vec(db_connection, users).await?;
                        let json: Json<Page<UserResource>> = resource_page.into();
                        Ok(json)
                    }
                    Err(e) => Err(e.into()),
                }
            }
        }
        .boxed()
    })
    .await
}

#[instrument(name = "user.api.find_all_by_identifiers", skip_all)]
pub async fn find_all_by_identifiers(
    Json(user_identifiers): Json<Vec<Uuid>>,
    Extension(context): Extension<DynContext>,
    Extension(authentication): Extension<DynAuthenticationHolder>,
) -> Result<Json<Vec<UserResource>>, AppError> {
    authentication.user_authenticated()?;
    transactional(context.db_connection(), |db_connection| {
        let identifiers = user_identifiers.clone();

        async move {
            match user_service::find_all_by_identifiers(db_connection, identifiers).await {
                Ok(users) => Ok(build_user_resources(db_connection, users).await?.into()),
                Err(e) => Err(e.into()),
            }
        }
        .boxed()
    })
    .await
}
