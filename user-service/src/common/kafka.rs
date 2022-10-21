use common_error::AppError;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;

use crate::config::configuration::KafkaConfiguration;

pub fn init_avro_encoder<'a, 'b>(
    config: &'a KafkaConfiguration,
) -> Result<AvroEncoder<'b>, AppError> {
    let sr_settings = resolve_sr_settings(config)?;
    Ok(get_avro_encoder(&sr_settings))
}

fn resolve_sr_settings(config: &KafkaConfiguration) -> Result<SrSettings, AppError> {
    Ok(SrSettings::new_builder(config.schema_registry.url.clone()).build()?)
}

fn get_avro_encoder<'a>(sr_settings: &SrSettings) -> AvroEncoder<'a> {
    AvroEncoder::new(sr_settings.clone())
}
