use tokio::task::JoinHandle;

pub async fn shutdown_signal(shutdown_handles: Vec<JoinHandle<()>>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Initialization of Ctrl+C handler failed");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Initialization of signal handler failed")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("Signal received, starting graceful shutdown");
    for handle in shutdown_handles {
        handle.abort();
    }
    opentelemetry::global::shutdown_tracer_provider();
}
