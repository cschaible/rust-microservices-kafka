use async_graphql::MergedObject;

use crate::user::api::graphql::mutation::types::user::UserInput;

pub mod types;

#[derive(MergedObject, Default)]
pub struct Mutation(UserInput);
