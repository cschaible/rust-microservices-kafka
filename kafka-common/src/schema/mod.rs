use avro_rs::Schema;
use avro_rs::Writer;
use schema_registry_converter::avro_common::DecodeResult;
use serde::Deserialize;
use serde::Serialize;

pub fn deserialize<'a, T: Deserialize<'a>>(
    decode_result: &'a DecodeResult,
) -> Result<T, avro_rs::Error> {
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

    avro_rs::from_value::<T>(&decode_result.value)
}

pub fn serialize<T: Serialize>(schema: &Schema, value: T) -> Result<Vec<u8>, avro_rs::Error> {
    let mut writer = Writer::new(schema, Vec::new());
    writer.append_ser(value)?;
    writer.into_inner()
}
