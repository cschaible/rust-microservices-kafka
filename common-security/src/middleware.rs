use axum::middleware;
use axum::Router;

use crate::authentication::auth;

pub trait RouterSecurityExt {
    fn add_auth_middleware(&self) -> Router;
}

impl RouterSecurityExt for Router {
    fn add_auth_middleware(&self) -> Self {
        self.clone().route_layer(middleware::from_fn(auth))
    }
}
