use async_trait::async_trait;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::ClientConfig;
use schema_registry_converter::async_impl::avro::AvroDecoder;
use schema_registry_converter::async_impl::avro::AvroEncoder;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::avro_common::DecodeResult;
use schema_registry_converter::error::SRCError;

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

pub fn get_avro_decoder<'a>(sr_settings: &SrSettings) -> AvroDecoder<'a> {
    AvroDecoder::new(sr_settings.clone())
}

// pub struct DeserializedRecord {}

#[async_trait]
pub trait RecordDecoder: Send + Sync {
    async fn decode(&self, bytes: Option<&[u8]>) -> Result<DecodeResult, SRCError>;
    // async fn deserialize<'b, T: Deserialize<'b>>(&self, value: Value) ->
    // Result<Box<T>, Error>;
}

pub struct AvroRecordDecoder<'a> {
    pub avro_decoder: AvroDecoder<'a>,
}

#[async_trait]
impl<'a> RecordDecoder for AvroRecordDecoder<'a> {
    async fn decode(&self, bytes: Option<&[u8]>) -> Result<DecodeResult, SRCError> {
        self.avro_decoder.decode(bytes).await
    }

    // async fn deserialize<'b, T: Deserialize<'b>>(&self, value: Value) ->
    // Result<T, Error> { avro_rs::from_value(&value)
    // }
}

pub fn init_consumer() -> StreamConsumer {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("auto.offset.reset", "earliest")
        .set("allow.auto.create.topics", "false")
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false") // Don't update offsets in store automatically
        .set("auto.commit.interval.ms", "5000") // Default
        .set("isolation.level", "read_committed")
        .set("max.poll.interval.ms", "60000")
        .set("request.timeout.ms", "10000")
        // Disable enable.auto.offset.store only if you want to disable auto commit
        //.set("enable.auto.offset.store", "false")
        .set("group.id", "accommodation-service1")
        .set("client.id", "accommodation-service")
        .set("metadata.max.age.ms", "60000")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    let topic = "user";
    consumer
        .subscribe(&[topic])
        .expect("Consumer topic subscription failed");

    consumer
}
