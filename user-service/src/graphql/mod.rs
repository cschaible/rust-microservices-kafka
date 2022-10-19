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

use crate::user::api::graphql::mutation::Mutation;
use crate::user::api::graphql::query::loaders::PhoneNumberLoader;
use crate::user::api::graphql::query::Query;
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
        DataLoader::new(PhoneNumberLoader::new(context.clone()), tokio::task::spawn);

    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
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
    let graphiql_port = if let Ok(port) = std::env::var("GRAPHIQL_PORT") {
        port
    } else {
        "3000".to_string()
    };

    response::Html(
        GraphiQLSource::build()
            .endpoint(&format!("http://localhost:{}/graphql", graphiql_port))
            .finish(),
    )
}
