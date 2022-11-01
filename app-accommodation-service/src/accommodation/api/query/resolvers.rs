use async_graphql::Context;
use async_graphql::Object;
use common_db_mongodb::transaction::transactional;
use common_error::AppError;

use crate::accommodation::api::query::types::accommodation::AccommodationPayload;
use crate::accommodation::api::shared::types::CountryCode;
use crate::accommodation::service::accommodation_service::find_accommodations;
use crate::DynContext;

#[derive(Default)]
pub struct AccommodationResolver;

#[Object]
impl AccommodationResolver {
    /// Get a list of accommodations.
    /// Accommodations can be filtered by name and country.
    pub async fn accommodations<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "optional name filter")] name: Option<String>,
        #[graphql(desc = "optional country filter")] country: Option<CountryCode>,
    ) -> Result<Vec<AccommodationPayload>, AppError> {
        let context = ctx.data_unchecked::<DynContext>();
        let accommodations = transactional(context.db_client(), |db_session| {
            let name_filter = name.clone();
            let country_filter = country;
            Box::pin(async move {
                let accommodations = find_accommodations(db_session, name_filter, country_filter)
                    .await?
                    .into_iter()
                    .map(AccommodationPayload)
                    .collect();

                Ok(accommodations)
            })
        })
        .await?;

        Ok(accommodations)
    }
}
