use std::collections::HashMap;

use async_trait::async_trait;
use common_error::AppError;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::ClientConfig;
use schema_registry_converter::async_impl::avro::AvroDecoder;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::avro_common::DecodeResult;
use schema_registry_converter::error::SRCError;

use crate::config::configuration::KafkaConfiguration;

pub fn init_avro_encoder<'a, 'b>(
    config: &'a KafkaConfiguration,
) -> Result<AvroEncoder<'b>, AppError> {
    let sr_settings = resolve_sr_settings(config)?;
    Ok(AvroEncoder::new(sr_settings))
}

pub fn init_avro_decoder<'a, 'b>(
    config: &'a KafkaConfiguration,
) -> Result<AvroDecoder<'b>, AppError> {
    let sr_settings = resolve_sr_settings(config)?;
    Ok(AvroDecoder::new(sr_settings))
}

pub fn resolve_sr_settings(config: &KafkaConfiguration) -> Result<SrSettings, AppError> {
    Ok(SrSettings::new_builder(config.schema_registry.url.clone()).build()?)
}

#[async_trait]
pub trait RecordDecoder: Send + Sync {
    async fn decode(&self, bytes: Option<&[u8]>) -> Result<DecodeResult, SRCError>;
}

pub struct AvroRecordDecoder<'a> {
    pub avro_decoder: AvroDecoder<'a>,
}

impl<'a> AvroRecordDecoder<'a> {
    pub fn new<'b>(config: &'b KafkaConfiguration) -> Result<AvroRecordDecoder<'a>, AppError> {
        Ok(AvroRecordDecoder {
            avro_decoder: init_avro_decoder(config)?,
        })
    }
}

#[async_trait]
impl<'a> RecordDecoder for AvroRecordDecoder<'a> {
    async fn decode(&self, bytes: Option<&[u8]>) -> Result<DecodeResult, SRCError> {
        self.avro_decoder.decode(bytes).await
    }
}

pub fn init_consumers(
    config: &KafkaConfiguration,
) -> Result<HashMap<String, StreamConsumer>, AppError> {
    let mut consumers = HashMap::new();
    for consumer_configuration in &config.consumer {
        // Initialize consumer
        let consumer = init_consumer(
            config.broker.urls.clone(),
            consumer_configuration.client_id.clone(),
            consumer_configuration.group_id.clone(),
            consumer_configuration.topic.clone(),
        )?;

        // Add consumer with id to the result map
        consumers.insert(consumer_configuration.id.clone(), consumer);
    }
    Ok(consumers)
}

fn init_consumer(
    bootstrap_servers: String,
    client_id: String,
    group_id: String,
    topics: Vec<String>,
) -> Result<StreamConsumer, AppError> {
    // Initialize consumer
    let consumer: StreamConsumer = ClientConfig::new()
        .set("auto.offset.reset", "earliest")
        .set("allow.auto.create.topics", "false")
        .set("bootstrap.servers", bootstrap_servers)
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false") // Don't update offsets in store automatically
        .set("auto.commit.interval.ms", "5000") // Default
        .set("isolation.level", "read_committed")
        .set("max.poll.interval.ms", "60000")
        .set("request.timeout.ms", "10000")
        .set("group.id", group_id)
        .set("client.id", client_id)
        .set("metadata.max.age.ms", "60000")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;

    // Convert topic names into &str
    let topics: Vec<&str> = topics.iter().map(|t| &**t).collect();

    // Subscribe to the specified topic names
    consumer.subscribe(&topics)?;

    Ok(consumer)
}
