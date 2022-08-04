use std::env;

use opentelemetry::sdk::{trace, Resource};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter};

pub fn init_tracing() {
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()),
    );

    // Copied from axum_tracing_opentelemetry::init_tracer_jaeger as it is not customizable

    use opentelemetry_semantic_conventions as semcov;
    let resource = Resource::new(vec![
        semcov::resource::SERVICE_NAME.string(env!("CARGO_PKG_NAME")),
        semcov::resource::SERVICE_VERSION.string(env!("CARGO_PKG_VERSION")),
    ]);

    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
    );

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(env!("CARGO_PKG_NAME"))
        .with_trace_config(
            trace::config()
                .with_resource(resource)
                .with_sampler(trace::Sampler::AlwaysOn),
        )
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Tracer initialization failed");

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer().json();

    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(EnvFilter::from_default_env())
        .with(otel_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Global subscriber initialization failed");
}
