use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SecurityConfiguration {
    pub jwks: JwksConfiguration,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct JwksConfiguration {
    pub issuer: String,
    pub url: String,
}
