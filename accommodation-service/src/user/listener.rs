use std::panic;
use std::sync::Arc;

use kafka_schema_common::schema_key::KeyAvro;
use kafka_schema_user::schema_create_user::CreateUserAvro;
use opentelemetry_propagator_b3::propagator::Propagator;
use opentelemetry_propagator_b3::propagator::B3_SINGLE_HEADER;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::BorrowedHeaders;
use rdkafka::message::FromBytes;
use rdkafka::message::Headers;
use rdkafka::Message;
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
use crate::user;
use crate::DynContext;

pub async fn listen(
    context: DynContext,
    stream_consumer: &StreamConsumer,
    tracing_propagator: Arc<Propagator>,
) {
    let decoder = context.avro_decoder().clone();

    loop {
        match stream_consumer.recv().await {
            Err(e) => warn!("Error: {}", e),
            Ok(message) => {
                // let span = span!(Level::TRACE, "accommodation_service.recv");
                // init_span_from_kafka_header(&span, message.headers(),
                // tracing_propagator.clone());

                let span = init_context(message.headers(), tracing_propagator.clone());
                let _ = span.enter();

                #[allow(clippy::unit_arg)]
                let _delivery_result: Instrumented<()> = {
                    let topic = message.topic();
                    assert_eq!(
                        topic, "user",
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
                    // warn!("trace_id: {}", trace_id);
                    let context =
                        get_context_from_b3(tracing_propagator.clone(), trace_id.to_string());
                    // span.set_parent_from_b3(tracing_propagator.clone(), trace_id.to_string());
                    tracing_opentelemetry::OpenTelemetrySpanExt::set_parent(&span, context);
                }
            }
        }
    }
    span
}

// fn init_span_from_kafka_header(
// span: &Span,
// headers: Option<&BorrowedHeaders>,
// tracing_propagator: Arc<Propagator>,
// ) {
// if let Some(headers) = headers {
// let header_count = headers.count();
// for i in 0..header_count {
// let header = headers.get(i).expect("Invalid header detected");
// if header.0 == B3_SINGLE_HEADER {
// if let Ok(trace_id) = str::from_bytes(header.1) {
// warn!("trace_id: {}", trace_id);
// span.set_parent_from_b3(tracing_propagator.clone(), trace_id.to_string());
// }
// }
// }
// }
// }

// fn trace_id_from_header(headers: Option<&BorrowedHeaders>) -> Option<String>
// { if let Some(headers) = headers {
// let header_count = headers.count();
// for i in 0..header_count {
// let header = headers.get(i).expect("Invalid header detected");
// if header.0 == B3_SINGLE_HEADER {
// if let Ok(trace_id) = str::from_bytes(header.1) {
// warn!("trace_id: {}", trace_id);
// return Some(trace_id.to_string());
// }
// }
// }
// }
// return None;
// }
