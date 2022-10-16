use axum::routing::get;
use axum::routing::post;
use axum::Router;

use super::create_user;
use super::find_all;
use super::find_all_by_identifiers;
use super::find_one_by_identifier;

pub fn init() -> Router {
    Router::new()
        .route("/users", get(find_all))
        .route("/users", post(create_user))
        .route("/users/:identifier", get(find_one_by_identifier))
        .route("/users/search", post(find_all_by_identifiers))
}
