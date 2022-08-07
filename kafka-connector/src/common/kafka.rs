use rdkafka::producer::{FutureProducer, Producer};
use rdkafka::util::Timeout;
use rdkafka::ClientConfig;
use std::time::Duration;

pub fn init_producer() -> FutureProducer {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("transactional.id", "kafka-connector")
        .set("queue.buffering.max.ms", "0") // Do not buffer
        .create()
        .expect("Producer creation failed");

    producer
        .init_transactions(Timeout::from(Duration::from_secs(30)))
        .expect("Transaction initialization failed");

    producer
}
