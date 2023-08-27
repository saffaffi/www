use axum::{routing::get, Router, Server};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use tracing::info;

mod handler;

#[tokio::main]
async fn main() {
    www_saffi::init_tracing();

    info!("starting server");

    let app = Router::new()
        .route("/", get(handler::index))
        .fallback(handler::not_found)
        .layer(OtelAxumLayer::default());

    Server::bind(&"0.0.0.0:4269".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
