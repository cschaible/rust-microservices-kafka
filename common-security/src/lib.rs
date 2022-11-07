extern crate core;

use jsonwebtoken::Algorithm;

use crate::config::SecurityConfiguration;
use crate::jwk::default_jwk::DefaultJwks;
use crate::jwk::error::JwkLoaderError;
use crate::jwk::JwkLoader;
use crate::jwt::default_jwt::DefaultJwt;
use crate::jwt::rsa::RsaJwtDecoder;
use crate::jwt::token::TokenDecoder;
use crate::jwt::token::TokenDecoders;

pub mod authentication;
pub mod config;
pub mod jwk;
pub mod jwt;
pub mod middleware;

pub extern crate jsonwebtoken;

pub async fn load_jwk_decoders(
    security_configuration: &SecurityConfiguration,
) -> Result<TokenDecoders, JwkLoaderError> {
    // Load JWKs from specified keycloak endpoint
    let jwk_loader: JwkLoader<DefaultJwks> =
        JwkLoader::from_url(security_configuration.jwks.url.clone()).await?;

    // Create token decoders for downloaded keys
    let mut jwk_decoders: Vec<Box<dyn TokenDecoder<DefaultJwt>>> = Vec::new();
    for jwk in jwk_loader.jwks.keys {
        jwk_decoders.push(Box::new(RsaJwtDecoder::new(Algorithm::RS256, vec![
            Box::new(jwk),
        ])));
    }

    // Return the decoders
    Ok(TokenDecoders {
        decoders: jwk_decoders,
    })
}
