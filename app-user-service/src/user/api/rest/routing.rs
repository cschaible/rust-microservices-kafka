use axum::routing::get;
use axum::routing::post;
use axum::Router;

use crate::user::api::rest::endpoints::create_user;
use crate::user::api::rest::endpoints::find_all;
use crate::user::api::rest::endpoints::find_all_by_identifiers;
use crate::user::api::rest::endpoints::find_one_by_identifier;

pub fn init() -> Router {
    Router::new()
        .route("/users", get(find_all))
        .route("/users", post(create_user))
        .route("/users/:identifier", get(find_one_by_identifier))
        .route("/users/search", post(find_all_by_identifiers))
}
