extern crate core;

use std::vec;

use clap::Parser;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;

use rdkafka::ClientConfig;
use tracing::warn;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Action to execute
    #[clap(value_enum)]
    action: TopicAction,

    /// Environment where to apply
    #[clap(value_enum)]
    env: Environment,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum TopicAction {
    CreateTopics,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Environment {
    Local,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let bootstrap_servers = match args.env {
        Environment::Local => "localhost:9092".to_owned(),
    };

    let admin_client: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .create()
        .expect("Admin client creation failed");

    match args.action {
        TopicAction::CreateTopics => {
            create_user_topic(&admin_client, 2, 1).await;
        }
    }
}

async fn create_user_topic(
    admin_client: &AdminClient<DefaultClientContext>,
    partitions: i32,
    replication_factor: i32,
) {
    create_topic(admin_client, "user", partitions, replication_factor).await;
}

async fn create_topic(
    admin_client: &AdminClient<DefaultClientContext>,
    topic: &str,
    partitions: i32,
    replication_factor: i32,
) {
    let topic_replication = TopicReplication::Fixed(replication_factor);
    let user_topic = NewTopic::new(topic, partitions, topic_replication);
    let topics = vec![&user_topic];

    let topic_result = admin_client
        .create_topics(topics, &AdminOptions::new())
        .await
        .unwrap_or_else(|_| panic!("Creation of topic \"{}\" failed", topic));

    topic_result.into_iter().for_each(|r| match r {
        Ok(t) => {
            println!(
                "Created topic: {} (partitions: {}, replication: {})",
                t, partitions, replication_factor
            );
        }
        Err(f) => {
            warn!("Creation of topic \"{}\" failed with reason: {}", f.0, f.1);
        }
    });
}
