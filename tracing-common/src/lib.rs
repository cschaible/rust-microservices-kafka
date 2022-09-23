use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use opentelemetry::sdk::trace;
use opentelemetry::sdk::Resource;
use opentelemetry::Context;
use opentelemetry_propagator_b3::propagator::B3Encoding;
use opentelemetry_propagator_b3::propagator::Propagator;
use opentelemetry_propagator_b3::propagator::B3_SINGLE_HEADER;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Initializes tracing.
///
/// Can be called as follows:  
/// init_tracing(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
pub fn init_tracing<T>(package_name: &str, package_version: &str, env_filter_customizer: T)
where T: Fn(EnvFilter) -> EnvFilter {
    env::set_var(
        "RUST_LOG",
        env::var("RUST_LOG").unwrap_or_else(|_| "warn".to_string()),
    );

    let resource: Resource = axum_tracing_opentelemetry::make_resource(
        package_name.to_string(),
        package_version.to_string(),
    );

    opentelemetry::global::set_text_map_propagator(
        // opentelemetry::sdk::propagation::TraceContextPropagator::new(),
        Propagator::with_encoding(B3Encoding::SingleHeader),
    );

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(package_name.to_string())
        .with_trace_config(
            trace::config()
                .with_resource(resource)
                .with_sampler(trace::Sampler::AlwaysOn),
        )
        .with_max_packet_size(9216) // Default max UDP packet size on macOs
        .with_auto_split_batch(true)
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Tracer initialization failed");

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer().json();

    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env_filter_customizer(
            EnvFilter::from_default_env().add_directive(
                "axum_tracing_opentelemetry=trace"
                    .parse()
                    .unwrap_or_default(),
            ),
        ))
        .with(otel_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Global subscriber initialization failed");
}

pub trait B3SpanExt {
    fn set_parent_from_b3(&self, propagator: Arc<Propagator>, b3_trace_id: String);
}

impl B3SpanExt for tracing::Span {
    fn set_parent_from_b3(&self, propagator: Arc<Propagator>, b3_trace_id: String) {
        let mut extractor: HashMap<String, String> = HashMap::new();
        extractor.insert(B3_SINGLE_HEADER.to_string(), b3_trace_id);

        let context = propagator
            .extract_single_header(&extractor)
            .expect("Couldn't extract trace header");

        use opentelemetry::trace::TraceContextExt;
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        self.set_parent(Context::new().with_remote_span_context(context));
    }
}

pub fn get_b3_trace_id() -> Option<String> {
    // Code partially taken from
    // axum_tracing_opentelemetry::find_current_trace_id();
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let context = tracing::Span::current().context();
    let span = context.span();
    let span_context = span.span_context();
    let span_id = span_context
        .is_valid()
        .then(|| span_context.span_id().to_string());

    let trace_id = span_context
        .is_valid()
        .then(|| span_context.trace_id().to_string());

    if trace_id.is_none() || span_id.is_none() {
        None
    } else {
        // https://github.com/openzipkin/b3-propagation
        Some(format!("{}-{}-1", trace_id.unwrap(), span_id.unwrap()))
    }
}
