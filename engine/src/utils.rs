use tokio::signal::ctrl_c;

/// Returns on shutdown signal
pub async fn shutdown_signal() {
    let ctrl_c = async {
        ctrl_c().await.expect("failed to listen for event");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }
}