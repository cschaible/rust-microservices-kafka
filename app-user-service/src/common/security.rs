use std::sync::Arc;

use async_trait::async_trait;
use common_db_relationaldb::transaction::transactional;
use common_error::AppError;
use common_security::authentication::DynTokenValidator;
use common_security::authentication::DynUserDetailsService;
use common_security::authentication::DynUserIdentifierExtractor;
use common_security::authentication::TokenValidator;
use common_security::authentication::UserDetails;
use common_security::authentication::UserDetailsService;
use common_security::authentication::UserIdentifierExtractor;
use common_security::config::SecurityConfiguration;
use common_security::jwt::default_jwt::DefaultJwt;
use common_security::jwt::error::TokenDecoderError;
use common_security::jwt::token::DynTokenDecoders;
use common_security::jwt::token::TokenDecoder;
use common_security::jwt::token::TokenDecoders;
use common_security::jwt::Claims;
use common_security::load_jwk_decoders;
use futures::FutureExt;
use uuid::Uuid;

use crate::common::context::DynContext;
use crate::user::service::user_service::find_one_by_identifier;

pub struct OAuthConfiguration {
    pub user_details_service: DynUserDetailsService,
    pub user_identifier_extractor: DynUserIdentifierExtractor,
    pub token_decoders: DynTokenDecoders,
    pub token_validator: DynTokenValidator,
}

impl OAuthConfiguration {
    pub async fn new(
        context: DynContext,
        security_configuration: &SecurityConfiguration,
    ) -> Result<OAuthConfiguration, AppError> {
        let token_decoders = load_jwk_decoders(security_configuration).await?;
        Ok(OAuthConfiguration {
            user_details_service: UserDetailsServiceImpl::new(context.clone()),
            user_identifier_extractor: UserIdentifierExtractorImpl::new(),
            token_decoders: TokenDecodersImpl::new(token_decoders),
            token_validator: TokenValidatorImpl::new(security_configuration),
        })
    }
}

pub struct UserIdentifierExtractorImpl {}

impl UserIdentifierExtractorImpl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> DynUserIdentifierExtractor {
        Arc::new(UserIdentifierExtractorImpl {})
    }
}

impl UserIdentifierExtractor for UserIdentifierExtractorImpl {
    fn extract(&self, token: &dyn Claims) -> Option<Uuid> {
        if let Some(token) = token.downcast_ref::<DefaultJwt>() {
            if let Some(sub) = &token.sub {
                if let Ok(identifier) = Uuid::parse_str(sub) {
                    return Some(identifier);
                };
            }
        }
        None
    }
}

pub struct UserDetailsServiceImpl {
    context: DynContext,
}

impl UserDetailsServiceImpl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: DynContext) -> DynUserDetailsService {
        Arc::new(UserDetailsServiceImpl { context })
    }
}

#[async_trait]
impl UserDetailsService for UserDetailsServiceImpl {
    async fn load_user(&self, identifier: Uuid) -> Option<Box<dyn UserDetails>> {
        let user = transactional(self.context.db_connection(), |tx| {
            async move { Ok(find_one_by_identifier(tx, identifier).await?) }.boxed()
        })
        .await;

        if let Ok(Some(u)) = user {
            Some(Box::new(u))
        } else {
            None
        }
    }
}

pub struct TokenDecodersImpl {
    pub jwk_decoders: Vec<Box<dyn TokenDecoder<DefaultJwt>>>,
}

impl TokenDecodersImpl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(token_decoders: TokenDecoders) -> DynTokenDecoders {
        Arc::new(token_decoders)
    }
}

pub struct TokenValidatorImpl {
    pub issuer: String,
}

impl TokenValidatorImpl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(security_configuration: &SecurityConfiguration) -> DynTokenValidator {
        Arc::new(TokenValidatorImpl {
            issuer: security_configuration.jwks.issuer.clone(),
        })
    }
}

impl TokenValidator for TokenValidatorImpl {
    fn validate(&self, claims: &dyn Claims) -> Result<(), TokenDecoderError> {
        if let Some(token) = claims.downcast_ref::<DefaultJwt>() {
            if let Some(issuer) = &token.iss {
                if &self.issuer == issuer {
                    return Ok(());
                }
            }
        }
        Err(TokenDecoderError::InvalidToken)
    }
}
