use axum::{body::Body, http::Request, routing::get, Router, Server};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    www_saffi::init_tracing();

    info!("starting server");

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                info!(route = %"/", "handling request");
                "Hello, wtf?!"
            }),
        )
        .fallback(|request: Request<Body>| {
            let uri = request.uri().clone();
            async move {
                warn!(route = %uri, "request received for unknown URI");
            }
        })
        .layer(OtelAxumLayer::default());

    Server::bind(&"0.0.0.0:4269".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
