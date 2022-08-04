use axum::{
    routing::{get, post},
    Router,
};

use super::{create_user, find_all, find_all_by_identifiers, find_one_by_identifier};

pub trait UserRouteExt {
    fn register_user_endpoints(&mut self) -> Self;
}

impl UserRouteExt for Router {
    fn register_user_endpoints(&mut self) -> Self {
        self.clone()
            .route("/users", get(find_all))
            .route("/users", post(create_user))
            .route("/users/:identifier", get(find_one_by_identifier))
            .route("/users/search", post(find_all_by_identifiers))
    }
}
