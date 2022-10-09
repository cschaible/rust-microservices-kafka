use async_graphql::MergedObject;
use async_graphql::Object;

use crate::accommodation::api::query::resolvers::AccommodationResolver;

pub mod loaders;
pub mod resolvers;
pub mod types;

#[derive(Default)]
pub struct SampleQuery;

#[Object]
impl SampleQuery {
    pub async fn howdy(&self) -> &'static str {
        "partner"
    }
}

#[derive(MergedObject, Default)]
pub struct Query(SampleQuery, AccommodationResolver);
