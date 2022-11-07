use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TokenDecoderError {
    /// The authentication data/token cannot be validated successfully.
    InvalidToken,

    /// The token is not valid anymore
    TokenExpired,
}

// Define IntoResponse as it is required by middleware route_layer.
impl IntoResponse for TokenDecoderError {
    fn into_response(self) -> Response {
        match self {
            TokenDecoderError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "Token expired").into_response()
            }
            TokenDecoderError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
            }
        }
    }
}
