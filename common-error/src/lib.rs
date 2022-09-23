use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::fmt::{Display, Formatter};

use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unhandled(anyhow::Error),
    UnhandledDbError(sea_orm::DbErr),
    MongoDbError(mongodb::error::Error),
    MongoDbBsonError(mongodb::bson::ser::Error),
    DbError(DbError),
    SerializationError(schema_registry_converter::error::SRCError),
}

#[derive(Debug)]
pub enum DbError {
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Map the error into error message, log message and status code
        let (error_message, log_message, status_code) = match_error(&self);

        // Log message depending on status code
        if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            // alternatively log "self" instead of the "log_message"
            tracing::error!("{:?}", log_message);
        } else if status_code == StatusCode::NOT_FOUND {
            tracing::debug!("{:?}", log_message);
        }

        // Build response body with error message
        let body = Json(json!({ "error": error_message }));

        (status_code, body).into_response()
    }
}

fn match_error(error: &AppError) -> (&str, String, StatusCode) {
    match error {
        AppError::Unhandled(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        AppError::UnhandledDbError(e) => match e {
            sea_orm::DbErr::Conn(err) => (
                "Internal Server Error",
                err.to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            sea_orm::DbErr::Exec(err) => (
                "Internal Server Error",
                err.to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            sea_orm::DbErr::Query(err) => (
                "Internal Server Error",
                err.to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            sea_orm::DbErr::RecordNotFound(err) => {
                ("Not found", err.to_owned(), StatusCode::NOT_FOUND)
            }
            sea_orm::DbErr::Custom(err) => (
                "Internal Server Error",
                err.to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            sea_orm::DbErr::Type(err) => (
                "Internal Server Error",
                err.to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            sea_orm::DbErr::Json(err) => (
                "Internal Server Error",
                err.to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            sea_orm::DbErr::Migration(err) => (
                "Internal Server Error",
                err.to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        },
        AppError::DbError(e) => match e {
            DbError::NotFound => (
                "Not found",
                "DB Entry not found".to_owned(),
                StatusCode::NOT_FOUND,
            ),
        },
        AppError::MongoDbError(e) => (
            "Internal Server Error",
            format!("{:?}", e.kind.as_ref()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        AppError::MongoDbBsonError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        AppError::SerializationError(e) => (
            "Internal Server Error",
            e.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Unhandled(e)
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(e: sea_orm::DbErr) -> Self {
        AppError::UnhandledDbError(e)
    }
}

impl From<schema_registry_converter::error::SRCError> for AppError {
    fn from(e: schema_registry_converter::error::SRCError) -> Self {
        AppError::SerializationError(e)
    }
}

impl From<mongodb::bson::ser::Error> for AppError {
    fn from(e: mongodb::bson::ser::Error) -> Self {
        AppError::MongoDbBsonError(e)
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        AppError::MongoDbError(e)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (error_message, _, _) = match_error(&self);
        write!(f, "{}", error_message)
    }
}
