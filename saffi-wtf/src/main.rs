use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router, Server,
};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use cfg_if::cfg_if;
use tokio::{signal, sync::RwLock};
use tracing::{info, warn};

use crate::templates::components::DynamicColours;

mod errors;
mod handlers;
mod templates;

#[derive(Clone, Debug, Default)]
pub struct AppState {
    colours: Arc<RwLock<DynamicColours>>,
}

#[tokio::main]
async fn main() {
    www_saffi::init_tracing();

    info!("starting server");

    let state = AppState::default();

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/style.css", get(handlers::stylesheet))
        .route("/api/make-green", post(handlers::make_green));

    #[cfg(debug_assertions)]
    let app = app.route("/break", get(handlers::internal_error));

    let app = app
        .fallback(handlers::not_found)
        .layer(OtelAxumLayer::default())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            errors::render_error,
        ))
        .with_state(state);

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
