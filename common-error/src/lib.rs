use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde_json::json;
use tracing::debug;
use tracing::error;

#[derive(Clone, Debug)]
pub enum AppError {
    #[cfg(feature = "security")]
    AuthenticationError(common_security::authentication::AuthenticationError),
    ConfigError(Arc<config::ConfigError>),
    DbError(DbError),
    IoError(Arc<std::io::Error>),
    #[cfg(feature = "security")]
    JwkLoaderError(common_security::jwk::error::JwkLoaderError),
    #[cfg(feature = "kafka")]
    KafkaError(rdkafka::error::KafkaError),
    #[cfg(feature = "mongodb")]
    MongoDbBsonError(mongodb::bson::ser::Error),
    #[cfg(feature = "mongodb")]
    MongoDbError(mongodb::error::Error),
    #[cfg(feature = "relationaldb")]
    RelDbUnhandledDbError(sea_orm::DbErr),
    #[cfg(feature = "scheduler")]
    SchedulerError(tokio_cron_scheduler::JobSchedulerError),
    #[cfg(feature = "kafka")]
    SerializationError(schema_registry_converter::error::SRCError),
    #[cfg(feature = "security")]
    TokenDecoderError(common_security::jwt::error::TokenDecoderError),
    Unhandled(Arc<anyhow::Error>),
}

#[derive(Clone, Debug)]
pub enum DbError {
    Conflict,
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Map the error into error message, log message and status code
        let (error_message, log_message, status_code) = match_error(&self);

        // Log message depending on status code
        if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            // alternatively log "self" instead of the "log_message"
            error!("{:?}", log_message);
        } else if status_code == StatusCode::NOT_FOUND {
            debug!("{:?}", log_message);
        }

        // Build response body with error message
        let body = Json(json!({ "error": error_message }));

        (status_code, body).into_response()
    }
}

pub fn match_error(error: &AppError) -> (&str, String, StatusCode) {
    match error {
        #[cfg(feature = "security")]
        AppError::AuthenticationError(e) => match e {
            common_security::authentication::AuthenticationError::AccessDenied => {
                ("Access denied", format!("{:?}", e), StatusCode::FORBIDDEN)
            }
            common_security::authentication::AuthenticationError::Unauthorized => {
                ("Unauthorized", format!("{:?}", e), StatusCode::UNAUTHORIZED)
            }
        },
        AppError::ConfigError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        AppError::DbError(e) => match e {
            DbError::Conflict => (
                "Conflict",
                "Outdated resource".to_owned(),
                StatusCode::CONFLICT,
            ),
            DbError::NotFound => (
                "Not found",
                "DB Entry not found".to_owned(),
                StatusCode::NOT_FOUND,
            ),
        },
        AppError::IoError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        #[cfg(feature = "security")]
        AppError::JwkLoaderError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        #[cfg(feature = "kafka")]
        AppError::KafkaError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        #[cfg(feature = "mongodb")]
        AppError::MongoDbBsonError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        #[cfg(feature = "mongodb")]
        AppError::MongoDbError(e) => (
            "Internal Server Error",
            format!("{:?}", e.kind.as_ref()),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        #[cfg(feature = "relationaldb")]
        AppError::RelDbUnhandledDbError(e) => handle_sea_orm_db_error(e),
        #[cfg(feature = "scheduler")]
        AppError::SchedulerError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        #[cfg(feature = "kafka")]
        AppError::SerializationError(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        #[cfg(feature = "security")]
        AppError::TokenDecoderError(e) => (
            "Invalid token",
            format!("{:?}", e),
            StatusCode::UNAUTHORIZED,
        ),
        AppError::Unhandled(e) => (
            "Internal Server Error",
            format!("{:?}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

#[cfg(feature = "relationaldb")]
fn handle_sea_orm_db_error(e: &sea_orm::DbErr) -> (&str, String, StatusCode) {
    match e {
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
        sea_orm::DbErr::RecordNotFound(err) => ("Not found", err.to_owned(), StatusCode::NOT_FOUND),
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
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Unhandled(Arc::new(e))
    }
}

impl From<config::ConfigError> for AppError {
    fn from(e: config::ConfigError) -> Self {
        AppError::ConfigError(Arc::new(e))
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::bson::ser::Error> for AppError {
    fn from(e: mongodb::bson::ser::Error) -> Self {
        AppError::MongoDbBsonError(e)
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        AppError::MongoDbError(e)
    }
}

#[cfg(feature = "kafka")]
impl From<rdkafka::error::KafkaError> for AppError {
    fn from(e: rdkafka::error::KafkaError) -> Self {
        AppError::KafkaError(e)
    }
}

#[cfg(feature = "kafka")]
impl From<schema_registry_converter::error::SRCError> for AppError {
    fn from(e: schema_registry_converter::error::SRCError) -> Self {
        AppError::SerializationError(e)
    }
}

#[cfg(feature = "security")]
impl From<common_security::authentication::AuthenticationError> for AppError {
    fn from(e: common_security::authentication::AuthenticationError) -> Self {
        AppError::AuthenticationError(e)
    }
}

#[cfg(feature = "security")]
impl From<common_security::jwk::error::JwkLoaderError> for AppError {
    fn from(e: common_security::jwk::error::JwkLoaderError) -> Self {
        AppError::JwkLoaderError(e)
    }
}

#[cfg(feature = "security")]
impl From<common_security::jwt::error::TokenDecoderError> for AppError {
    fn from(e: common_security::jwt::error::TokenDecoderError) -> Self {
        AppError::TokenDecoderError(e)
    }
}

#[cfg(feature = "relationaldb")]
impl From<sea_orm::DbErr> for AppError {
    fn from(e: sea_orm::DbErr) -> Self {
        AppError::RelDbUnhandledDbError(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::IoError(Arc::new(e))
    }
}

#[cfg(feature = "scheduler")]
impl From<tokio_cron_scheduler::JobSchedulerError> for AppError {
    fn from(e: tokio_cron_scheduler::JobSchedulerError) -> Self {
        AppError::SchedulerError(e)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (error_message, log_message, status_code) = match_error(self);
        // This is a workaround to log error details from graphql requests
        if status_code == StatusCode::INTERNAL_SERVER_ERROR {
            error!("{:?}", log_message);
        }
        write!(f, "{}", error_message)
    }
}
