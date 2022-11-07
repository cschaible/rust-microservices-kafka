use jsonwebtoken::decode;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use serde::Deserialize;

use crate::jwk::RsaKeyComponents;
use crate::jwt::error::TokenDecoderError;
use crate::jwt::token::TokenDecoder;
use crate::jwt::Claims;

/// RSA JWT decoder trait definition.
#[derive(Clone)]
pub struct RsaJwtDecoder {
    algorithm: Algorithm,
    decoding_keys: Vec<DecodingKey>,
}

impl RsaJwtDecoder {
    /// Constructs a new instance of `RsaJwtDecoder` for the given algorithm and
    /// keys.
    pub fn new(algorithm: Algorithm, rsa_keys: Vec<Box<dyn RsaKeyComponents>>) -> RsaJwtDecoder {
        let mut decoding_keys: Vec<DecodingKey> = Vec::new();

        for rsa_key in &rsa_keys {
            let n: String = rsa_key.get_n().expect("rsa n component not found");
            let e: String = rsa_key.get_e().expect("rsa e component not found");
            let decoding_key = DecodingKey::from_rsa_components(n.as_ref(), e.as_ref())
                .expect("Decoding of JWKs failed");
            decoding_keys.push(decoding_key);
        }

        RsaJwtDecoder {
            algorithm,
            decoding_keys,
        }
    }
}

impl<T: for<'b> Deserialize<'b> + Claims> TokenDecoder<T> for RsaJwtDecoder {
    fn decode_token(&self, token: &str) -> Result<Box<T>, TokenDecoderError> {
        for key in &self.decoding_keys {
            let result = decode::<T>(token, key, &Validation::new(self.algorithm));
            match result {
                Ok(decoded_token) => return Ok(Box::new(decoded_token.claims)),
                Err(e) => {
                    if let ErrorKind::ExpiredSignature = e.kind() {
                        return Err(TokenDecoderError::TokenExpired);
                    } // Otherwise try next key
                }
            }
        }
        Err(TokenDecoderError::InvalidToken)
    }
}
