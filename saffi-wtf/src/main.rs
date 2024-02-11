use axum::{middleware, routing::get, Router, Server};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use tracing::info;

mod errors;
mod handlers;
mod templates;

#[derive(Clone, Debug, Default)]
pub struct AppState {}

#[tokio::main]
async fn main() {
    www_saffi::init_tracing();

    let addr = "0.0.0.0:4269".parse().unwrap();

    info!(%addr, "starting server");

    let state = AppState::default();

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/style.css", get(handlers::stylesheet));

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

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(www_saffi::graceful_shutdown())
        .await
        .unwrap();
}
