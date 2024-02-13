use cfg_if::cfg_if;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() {
    #[cfg(debug_assertions)]
    let fmt_layer = fmt::layer().with_timer(fmt::time::uptime()).pretty();
    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt::layer();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("otel::tracing=trace,info"))
                .unwrap(),
        )
        .with(fmt_layer)
        .init();
}

pub async fn graceful_shutdown() {
    // A future that will listen for the ctrl-c input from a terminal.
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("should be able to listen for ctrl-c event");
    };

    cfg_if! {
        if #[cfg(unix)] {
            // A future that will listen for a SIGTERM signal.
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("should be able to install signal handler")
                    .recv()
                    .await;
            };
        } else {
            // A future that will never complete, because non-Unix platforms
            // don't have Unix signals!
            let terminate = std::future::pending::<()>();
        }
    };

    // Wait for either of those futures to complete, which means that one of the
    // termination signals has been received.
    tokio::select! {
        _ = ctrl_c => info!("ctrl-c received, starting graceful shutdown"),
        _ = terminate => info!("termination signal received, starting graceful shutdown"),
    }

    info!("finished shutting down; see you soon!");
}
