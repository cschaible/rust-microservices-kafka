use anyhow::Result;
use sea_orm::ConnectionTrait;

use self::user_resource::UserResource;
use crate::common::paging::Page;
use crate::user::model::phone_number;
use crate::user::model::user;
use crate::user::service::phone_number_service;

pub mod phone_number_resource;
pub mod user_resource;

pub async fn build_user_resource<T: ConnectionTrait + Sized>(
    connection: &T,
    user: user::Model,
) -> Result<UserResource> {
    Ok(build_resources(connection, vec![user])
        .await?
        .first()
        .expect("User resource conversion failed")
        .to_owned())
}

pub async fn build_user_resources<T: ConnectionTrait + Sized>(
    connection: &T,
    users: Vec<user::Model>,
) -> Result<Vec<UserResource>> {
    build_resources(connection, users).await
}

pub async fn build_user_resource_page_from_page<T: ConnectionTrait + Sized>(
    connection: &T,
    user_page: Page<user::Model>,
) -> Result<Page<UserResource>> {
    let user_resources = build_resources(connection, user_page.items).await?;

    Ok(Page {
        items: user_resources,
        page: user_page.page,
        size: user_page.size,
        total_elements: user_page.total_elements,
        total_pages: user_page.total_pages,
    })
}

pub async fn build_user_resource_page_from_vec<T: ConnectionTrait + Sized>(
    connection: &T,
    users: Vec<user::Model>,
) -> Result<Page<UserResource>> {
    let user_resources = build_resources(connection, users).await?;

    Ok(Page {
        items: user_resources,
        page: None,
        size: None,
        total_elements: None,
        total_pages: None,
    })
}

async fn build_resources<T: ConnectionTrait + Sized>(
    connection: &T,
    users: Vec<user::Model>,
) -> Result<Vec<UserResource>> {
    let user_identifiers = users.iter().map(|u| u.identifier).collect();
    let phone_numbers_by_user_id =
        phone_number_service::find_all_by_user_identifiers(connection, user_identifiers).await?;

    let user_resources: Vec<UserResource> = users
        .into_iter()
        .map(|u| {
            // Get the phone numbers of the user from the map
            let phone_numbers: Option<Vec<phone_number::Model>> =
                phone_numbers_by_user_id.get(&u.id).map(|p| p.to_owned());

            // Convert the user and the phone_numbers into a resource
            (u, phone_numbers).into()
        })
        .collect();

    Ok(user_resources)
}
