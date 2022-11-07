pub mod default_jwk;
pub mod error;

use std::fs;

use error::JwkLoaderError;
use serde::Deserialize;

pub trait RsaKeyComponents {
    fn get_n(&self) -> Option<String>;
    fn get_e(&self) -> Option<String>;
}

/// JWK loader definition
pub struct JwkLoader<T: for<'a> Deserialize<'a>> {
    pub jwks: T,
}

impl<T: for<'a> Deserialize<'a>> JwkLoader<T> {
    /// Load a JWK file from disk.
    pub fn from_file(filename: String) -> Result<JwkLoader<T>, JwkLoaderError> {
        match fs::read_to_string(filename) {
            Ok(key) => match serde_json::from_str(key.as_str()) {
                Ok(jwks) => Ok(JwkLoader { jwks }),
                Err(_) => Err(JwkLoaderError::InvalidKeyFile),
            },
            Err(_) => Err(JwkLoaderError::KeyFileCouldNotBeRead),
        }
    }

    /// Download a JWK file from a remote location with http.
    pub async fn from_url(url: String) -> Result<JwkLoader<T>, JwkLoaderError> {
        match reqwest::get(&url).await {
            Ok(response) => match response.json::<T>().await {
                Ok(jwks) => Ok(JwkLoader { jwks }),
                Err(_) => Err(JwkLoaderError::InvalidJsonResponse),
            },
            Err(_) => Err(JwkLoaderError::JwkDownloadFailed),
        }
    }
}
