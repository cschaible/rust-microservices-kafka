use async_graphql::MergedObject;

use crate::accommodation::api::query::resolvers::AccommodationResolver;

pub mod loaders;
pub mod resolvers;
pub mod types;

#[derive(MergedObject, Default)]
pub struct Query(AccommodationResolver);
