use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    Unhandled(anyhow::Error),
    UnhandledDbError(sea_orm::DbErr),
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
        let (error_message, log_message, status_code) = match &self {
            AppError::Unhandled(e) => (
                "Internal Server Error",
                format!("{:?}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            AppError::UnhandledDbError(e) => match e {
                DbErr::Conn(err) => (
                    "Internal Server Error",
                    err.to_owned(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
                DbErr::Exec(err) => (
                    "Internal Server Error",
                    err.to_owned(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
                DbErr::Query(err) => (
                    "Internal Server Error",
                    err.to_owned(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
                DbErr::RecordNotFound(err) => ("Not found", err.to_owned(), StatusCode::NOT_FOUND),
                DbErr::Custom(err) => (
                    "Internal Server Error",
                    err.to_owned(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
                DbErr::Type(err) => (
                    "Internal Server Error",
                    err.to_owned(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
                DbErr::Json(err) => (
                    "Internal Server Error",
                    err.to_owned(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
                DbErr::Migration(err) => (
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
            AppError::SerializationError(e) => (
                "Internal Server Error",
                e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };

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

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Unhandled(e)
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        AppError::UnhandledDbError(e)
    }
}

impl From<schema_registry_converter::error::SRCError> for AppError {
    fn from(e: schema_registry_converter::error::SRCError) -> Self {
        AppError::SerializationError(e)
    }
}
