use std::net::SocketAddr;

use axum::{body::Body, http::Request, middleware, routing::get, Router};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use tokio::net::TcpListener;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::info;

mod errors;
mod handlers;
mod templates;

#[derive(Clone, Debug, Default)]
pub struct AppState {}

#[tokio::main]
async fn main() {
    www_saffi::init_tracing();

    let state = AppState::default();

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/style.css", get(handlers::stylesheet));

    let app = app.nest_service(
        "/static",
        ServeDir::new("saffi-wtf/static").map_request(|req: Request<Body>| {
            info!(route = %req.uri(), under = %"/static", "handling nested request");
            req
        }),
    );

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

    let addr: SocketAddr = "0.0.0.0:4269".parse().unwrap();
    info!(%addr, "starting server");

    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(www_saffi::graceful_shutdown())
        .await
        .unwrap();
}
