use crate::{IsoCountryCodeEnumAvro, PhoneNumberAvro};
use serde::{Deserialize, Serialize};

pub const SCHEMA_NAME_CREATE_USER: &str = "create_user";

pub const RAW_SCHEMA_CREATE_USER_V1: &str = r#"
    {
        "name": "user",
        "type": "record",
        "fields": [
            {
                "name": "identifier",
                "type": "string"
            },
            {
                "name": "name",
                "type": "string"
            },
            {
                "name": "email",
                "type": "string"
            },
            {
                "name": "country",
                "type": {
                    "name": "IsoCountryCodeEnumAvro",
                    "symbols": [
                        "DE",
                        "US"
                    ],
                    "type": "enum"
                }
            },
            {
                "name": "phoneNumbers",
                "type": {
                    "type": "array",
                    "items": {
                        "name": "phoneNumber",
                        "type": "record",
                        "fields": [
                            {
                                "name": "countryCode",
                                "type": "string"
                            },
                            {
                                "name": "phoneNumberType",
                                "type": {
                                    "name": "PhoneNumberTypeEnumAvro",
                                    "symbols": [
                                        "Business",
                                        "Home",
                                        "Mobile"
                                    ],
                                    "type": "enum"
                                }
                            },
                            {
                                "name": "callNumber",
                                "type": "string"
                            }
                        ]
                    }
                }
            }
        ]
    }
"#;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserAvro {
    pub identifier: String,
    pub name: String,
    pub email: String,
    pub country: IsoCountryCodeEnumAvro,
    pub phone_numbers: Vec<PhoneNumberAvro>,
}
