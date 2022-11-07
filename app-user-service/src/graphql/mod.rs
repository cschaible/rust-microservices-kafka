use std::sync::atomic::Ordering::SeqCst;

use async_graphql::dataloader::DataLoader;
use async_graphql::http::GraphiQLSource;
use async_graphql::EmptySubscription;
use async_graphql::Schema;
use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;
use axum::response;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Extension;
use axum::Router;
use common_security::authentication::DynAuthenticationHolder;

use crate::common::api;
use crate::user::api::graphql::mutation::Mutation;
use crate::user::api::graphql::query::loaders::PhoneNumberLoader;
use crate::user::api::graphql::query::Query;
use crate::DynContext;

pub type ApplicationSchema = Schema<Query, Mutation, EmptySubscription>;

async fn graphql_handler(
    Extension(authentication): Extension<DynAuthenticationHolder>,
    schema: Extension<ApplicationSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.0.data(authentication)).await.into()
}

pub fn routing(context: DynContext) -> Router {
    let phone_number_loader =
        DataLoader::new(PhoneNumberLoader::new(context.clone()), tokio::task::spawn);

    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .enable_federation()
        .data(context)
        .data(phone_number_loader)
        // TODO check if "Tracing" extension can log errors properly
        //.extension(Tracing)
        .finish();

    Router::new()
        .route("/graphql", get(graphql_sdl).post(graphql_handler))
        .route("/ui", get(graphiql))
        .layer(Extension(schema))
}

async fn graphql_sdl(schema: Extension<ApplicationSchema>) -> impl IntoResponse {
    schema.sdl()
}

async fn graphiql() -> impl IntoResponse {
    let port = api::SERVER_PORT.load(SeqCst);
    response::Html(
        GraphiQLSource::build()
            .endpoint(&format!("http://localhost:{}/graphql", port))
            .finish(),
    )
}
