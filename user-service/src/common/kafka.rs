use schema_registry_converter::async_impl::{avro::AvroEncoder, schema_registry::SrSettings};

pub fn resolve_sr_settings() -> SrSettings {
    let schema_registry_url = std::env::var("KAFKA_SCHEMA_REGISTRY_URL")
        .expect("Environment variable 'KAFKA_SCHEMA_REGISTRY_URL' not set");

    SrSettings::new_builder(schema_registry_url)
        .build()
        .expect("Initialization of schema registry configuration failes")
}

pub fn get_avro_encoder<'a>(sr_settings: &SrSettings) -> AvroEncoder<'a> {
    AvroEncoder::new(sr_settings.clone())
}
