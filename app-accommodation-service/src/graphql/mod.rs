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

use crate::accommodation::api::mutation::Mutation;
use crate::accommodation::api::query::loaders::RoomTypeLoader;
use crate::accommodation::api::query::Query;
use crate::common::api;
use crate::DynContext;

pub type ApplicationSchema = Schema<Query, Mutation, EmptySubscription>;

async fn graphql_handler(
    schema: Extension<ApplicationSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub fn routing(context: DynContext) -> Router {
    let room_type_loader =
        DataLoader::new(RoomTypeLoader::new(context.clone()), tokio::task::spawn);

    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .enable_federation()
        .data(context)
        .data(room_type_loader)
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
