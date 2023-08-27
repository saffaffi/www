use axum::{routing::get, Router, Server};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use cfg_if::cfg_if;
use tokio::signal;
use tracing::{info, warn};

mod handlers;

#[tokio::main]
async fn main() {
    www_saffi::init_tracing();

    info!("starting server");

    let app = Router::new()
        .route("/", get(handlers::index))
        .fallback(handlers::not_found)
        .layer(OtelAxumLayer::default());

    Server::bind(&"0.0.0.0:4269".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
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
                    // A future that will never complete, because non-Unix
                    // platforms don't have Unix signals!
                    let terminate = std::future::pending::<()>();
                }
            };

            // Wait for either of those futures to complete, which means that
            // one of the termination signals has been received.
            tokio::select! {
                _ = ctrl_c => warn!("ctrl-c received, starting graceful shutdown"),
                _ = terminate => warn!("termination signal received, starting graceful shutdown"),
            }

            info!("finished shutting down; see you soon!");
        })
        .await
        .unwrap();
}
