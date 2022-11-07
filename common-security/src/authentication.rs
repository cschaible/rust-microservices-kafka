use std::sync::Arc;

use async_trait::async_trait;
use axum::http;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use downcast_rs::impl_downcast;
use downcast_rs::Downcast;
use uuid::Uuid;

use crate::jwt::error::TokenDecoderError;
use crate::jwt::token::DynTokenDecoders;
use crate::jwt::Claims;

pub trait UserDetails: Downcast + Sync + Send {}
impl_downcast!(UserDetails);

#[async_trait]
pub trait UserDetailsService: Send + Sync {
    async fn load_user(&self, identifier: Uuid) -> Option<Box<dyn UserDetails>>;
}

pub type DynUserDetailsService = Arc<dyn UserDetailsService>;

pub trait UserIdentifierExtractor: Send + Sync {
    fn extract(&self, decoded_token: &dyn Claims) -> Option<Uuid>;
}

pub type DynUserIdentifierExtractor = Arc<dyn UserIdentifierExtractor>;

pub trait TokenValidator: Send + Sync {
    fn validate(&self, claims: &dyn Claims) -> Result<(), TokenDecoderError>;
}

pub type DynTokenValidator = Arc<dyn TokenValidator>;

pub enum AuthenticationHolder {
    AuthenticatedUser(Authentication),
    AuthenticatedNewUser(NewUserAuthentication),
    NotAuthenticated,
}

pub type DynAuthenticationHolder = Arc<AuthenticationHolder>;

impl AuthenticationHolder {
    pub fn user_authenticated(&self) -> Result<&Authentication, AuthenticationError> {
        match self {
            AuthenticationHolder::AuthenticatedUser(u) => Ok(u),
            AuthenticationHolder::AuthenticatedNewUser(_) => Err(AuthenticationError::AccessDenied),
            _ => Err(AuthenticationError::Unauthorized),
        }
    }

    pub fn new_user_authenticated(&self) -> Result<&NewUserAuthentication, AuthenticationError> {
        match self {
            AuthenticationHolder::AuthenticatedUser(_) => Err(AuthenticationError::AccessDenied),
            AuthenticationHolder::AuthenticatedNewUser(u) => Ok(u),
            _ => Err(AuthenticationError::Unauthorized),
        }
    }
}

pub struct Authentication {
    pub token: String,
    pub principal: Box<dyn UserDetails>,
}

pub struct NewUserAuthentication {
    pub token: String,
    pub user_identifier: Uuid,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum AuthenticationError {
    AccessDenied,
    Unauthorized,
}

pub async fn auth<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, TokenDecoderError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        let auth_header_length = auth_header.len();
        if auth_header_length > 10
            && auth_header_length < 8 * 1024 * 1024
            && auth_header.starts_with("Bearer ")
        {
            // Extract token from header string
            let auth_header = auth_header[7..].as_ref();

            // Get dependencies from request context
            let user_details_service = req
                .extensions()
                .get::<DynUserDetailsService>()
                .expect("UserDetailsService not found in request context");

            let user_identifier_extractor = req
                .extensions()
                .get::<DynUserIdentifierExtractor>()
                .expect("UserIdentifierExtractor not found in request context");

            let token_decoders = req
                .extensions()
                .get::<DynTokenDecoders>()
                .expect("TokenDecoders not found in request context");

            let token_validator = req
                .extensions()
                .get::<DynTokenValidator>()
                .expect("TokenValidator not found in request context");

            // Decode claims from token
            let decoded_token = decode_token(token_decoders, auth_header)?;

            // Validate the token
            token_validator.validate(decoded_token.as_ref())?;

            // Authenticate the current user
            let auth = authenticate_current_user(
                user_details_service,
                user_identifier_extractor,
                auth_header,
                decoded_token.as_ref(),
            )
            .await;

            if let Some(auth) = auth {
                req.extensions_mut().insert(Arc::new(auth));
            } else {
                return Err(TokenDecoderError::InvalidToken);
            }
        } else {
            return Err(TokenDecoderError::InvalidToken);
        }
    } else {
        // If no authorization header is provided return "NotAuthenticated"
        req.extensions_mut()
            .insert(Arc::new(AuthenticationHolder::NotAuthenticated));
    };

    // Proceed with the request chain
    Ok(next.run(req).await)
}

async fn authenticate_current_user(
    user_details_service: &DynUserDetailsService,
    user_identifier_extractor: &DynUserIdentifierExtractor,
    auth_token: &str,
    decoded_token: &dyn Claims,
) -> Option<AuthenticationHolder> {
    if let Some(user_identifier) = user_identifier_extractor.extract(decoded_token) {
        if let Some(user) = user_details_service.load_user(user_identifier).await {
            Some(AuthenticationHolder::AuthenticatedUser(Authentication {
                token: auth_token.to_string(),
                principal: user,
            }))
        } else {
            Some(AuthenticationHolder::AuthenticatedNewUser(
                NewUserAuthentication {
                    token: auth_token.to_string(),
                    user_identifier,
                },
            ))
        }
    } else {
        None
    }
}

fn decode_token(
    token_decoders: &DynTokenDecoders,
    auth_header: &str,
) -> Result<Box<dyn Claims>, TokenDecoderError> {
    for decoder in &token_decoders.decoders {
        match decoder.decode_token(auth_header) {
            Ok(decoded_token) => return Ok(decoded_token),
            Err(e) => {
                if e == TokenDecoderError::TokenExpired {
                    return Err(e);
                } // Otherwise try next decoder
            }
        }
    }
    Err(TokenDecoderError::InvalidToken)
}
