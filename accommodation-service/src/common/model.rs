use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum IsoCountryCodeEnum {
    DE,
    US,
}

impl IsoCountryCodeEnum {
    pub fn country_name(&self) -> String {
        match self {
            IsoCountryCodeEnum::DE => "Germany".to_string(),
            IsoCountryCodeEnum::US => "United States of America".to_string(),
        }
    }
}
