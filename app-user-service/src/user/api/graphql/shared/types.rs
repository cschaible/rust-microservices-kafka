use async_graphql::Enum;

use crate::common::model::IsoCountryCodeEnum;

/// The country where the accommodation is.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CountryCode {
    /// Germany
    DE,

    /// United States
    US,
}

impl From<IsoCountryCodeEnum> for CountryCode {
    fn from(iso_country_code: IsoCountryCodeEnum) -> Self {
        match iso_country_code {
            IsoCountryCodeEnum::DE => CountryCode::DE,
            IsoCountryCodeEnum::US => CountryCode::US,
        }
    }
}

impl From<CountryCode> for IsoCountryCodeEnum {
    fn from(country_code: CountryCode) -> Self {
        match country_code {
            CountryCode::DE => IsoCountryCodeEnum::DE,
            CountryCode::US => IsoCountryCodeEnum::US,
        }
    }
}

/// Types of phone numbers.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PhoneNumberType {
    /// Business number
    Business,

    /// Home number
    Home,

    /// Mobile number
    Mobile,
}
