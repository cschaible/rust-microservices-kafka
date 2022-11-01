use async_graphql::MergedObject;

use crate::user::api::graphql::query::resolvers::UserResolver;

pub mod loaders;
pub mod resolvers;
pub mod types;

#[derive(MergedObject, Default)]
pub struct Query(UserResolver);
