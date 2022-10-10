use apache_avro::Schema;
use apache_avro::Writer;
use schema_registry_converter::avro_common::DecodeResult;
use serde::Deserialize;
use serde::Serialize;

pub fn deserialize<'a, T: Deserialize<'a>>(
    decode_result: &'a DecodeResult,
) -> Result<T, apache_avro::Error> {
    match decode_result.name.clone() {
        Some(name) => {
            if name.name != "user" {
                panic!(
                    "User cannot be decoded. Provided data of type {}",
                    name.name
                )
            }
        }
        _ => panic!("Unknown type cannot be decoded."),
    };

    apache_avro::from_value::<T>(&decode_result.value)
}

pub fn serialize<T: Serialize>(schema: &Schema, value: T) -> Result<Vec<u8>, apache_avro::Error> {
    let mut writer = Writer::new(schema, Vec::new());
    writer.append_ser(value)?;
    writer.into_inner()
}
