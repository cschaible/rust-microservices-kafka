use async_graphql::MergedObject;

use crate::accommodation::api::mutation::types::accommodation::AccommodationInput;
use crate::accommodation::api::mutation::types::room_type::RoomTypeInput;

pub mod types;

#[derive(MergedObject, Default)]
pub struct Mutation(AccommodationInput, RoomTypeInput);
