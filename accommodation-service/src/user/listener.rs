use std::panic;
use std::sync::Arc;

use apache_avro::schema::Name;
use kafka_schema_common::schema_key::KeyAvro;
use kafka_schema_user::schema_create_user::CreateUserAvro;
use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;
use opentelemetry_propagator_b3::propagator::Propagator;
use opentelemetry_propagator_b3::propagator::B3_SINGLE_HEADER;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::BorrowedHeaders;
use rdkafka::message::FromBytes;
use rdkafka::message::Headers;
use rdkafka::Message;
use tokio::task::JoinHandle;
use tracing::debug;
use tracing::instrument::Instrumented;
use tracing::span;
use tracing::warn;
use tracing::Instrument;
use tracing::Level;
use tracing::Span;
use tracing_common::get_context_from_b3;
// use tracing_common::B3SpanExt;
use uuid::Uuid;

use crate::common::db::transactional2;
use crate::config::configuration::ConsumerConfiguration;
use crate::config::configuration::KafkaConfiguration;
use crate::user;
use crate::DynContext;

pub fn listen(
    context: DynContext,
    config: &KafkaConfiguration,
    stream_consumer: StreamConsumer,
    tracing_propagator: Arc<Propagator>,
) -> JoinHandle<()> {
    let topic = get_user_topic_name(config);

    // Start listener
    tokio::spawn(async move {
        do_listen(context, &stream_consumer, topic, tracing_propagator).await;
    })
}

pub async fn do_listen(
    context: DynContext,
    stream_consumer: &StreamConsumer,
    user_topic: String,
    tracing_propagator: Arc<Propagator>,
) {
    let decoder = context.avro_decoder().clone();

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
                let span = init_context(message.headers(), tracing_propagator.clone());
                let _ = span.enter();

                #[allow(clippy::unit_arg)]
                let _delivery_result: Instrumented<()> = {
                    let topic = message.topic();
                    assert_eq!(
                        topic, user_topic,
                        "Message from wrong topic detected. Stopped processing."
                    );

                    let key_result = decoder
                        .decode(message.key())
                        .await
                        .expect("Couldn't decode avro message");

                    let key = apache_avro::from_value::<KeyAvro>(&key_result.value)
                        .expect("Couldn't deserialize KeyAvro");

                    let payload_result = decoder
                        .decode(message.payload())
                        .await
                        .expect("Couldn't decode payload");

                    // Check type
                    assert_eq!(
                        payload_result
                            .name
                            .unwrap_or_else(|| Name {
                                name: "".to_string(),
                                namespace: None
                            })
                            .name,
                        SCHEMA_NAME_CREATE_USER.to_string()
                    );

                    let payload = apache_avro::from_value::<CreateUserAvro>(&payload_result.value)
                        .expect("Couldn't deserialize CreateUserAvro");

                    debug!(
                        "Message received (topic: {}, partition: {}, offset: {}",
                        topic,
                        message.partition(),
                        message.offset()
                    );

                    // debug!("Key: {:?}", key);
                    // debug!("Value: {:?}", payload);

                    if let Err(e) = transactional2(context.clone(), |tx_context| {
                        let identifier: Uuid = payload.identifier.parse().expect("Invalid UUID");
                        let version: i64 = key.identifier.version;
                        let name: String = payload.name.clone();

                        Box::pin(async move {
                            match user::service::create_user(tx_context, identifier, version, name)
                                .await
                            {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e),
                            }
                        })
                    })
                    .instrument(span.clone())
                    .await
                    {
                        panic!("Consumption of event failed: {:?}", e);
                    }

                    if let Err(e) = stream_consumer.store_offset_from_message(&message) {
                        panic!("Error while storing offset: {}", e);
                    }
                }
                .instrument(span);
            }
        }
    }
}

fn init_context(headers: Option<&BorrowedHeaders>, tracing_propagator: Arc<Propagator>) -> Span {
    let span = span!(Level::TRACE, "");
    if let Some(headers) = headers {
        let header_count = headers.count();
        for i in 0..header_count {
            let header = headers.get(i).expect("Invalid header detected");
            if header.0 == B3_SINGLE_HEADER {
                if let Ok(trace_id) = str::from_bytes(header.1) {
                    let context =
                        get_context_from_b3(tracing_propagator.clone(), trace_id.to_string());
                    tracing_opentelemetry::OpenTelemetrySpanExt::set_parent(&span, context);
                }
            }
        }
    }
    span
}

fn get_user_topic_name(config: &KafkaConfiguration) -> String {
    // Get consumer configuration
    let consumer_config: Vec<&ConsumerConfiguration> = config
        .consumer
        .iter()
        .filter(|c| c.id.clone() == "user")
        .collect();

    // Get topic name
    let topic = consumer_config
        .first()
        .expect("user consumer configuration not found")
        .topic
        .clone()
        .first()
        .expect("user topic not found in consumer configuration")
        .clone();

    topic
}
