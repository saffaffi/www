use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::info;

/// Errors that can be returned by request handlers.
#[derive(Error, Debug)]
pub enum HandlerError {
    /// The requested page was not found.
    #[error("page not found")]
    NotFound,
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(match self {
                HandlerError::NotFound => StatusCode::NOT_FOUND,
            })
            .body(body::boxed(body::Empty::new()))
            .unwrap()
    }
}

pub async fn index() -> Result<String, HandlerError> {
    info!(route = %"/", "handling request");
    Ok("Hello, wtf?!".into())
}

pub async fn not_found(request: Request<Body>) -> HandlerError {
    let uri = request.uri();
    info!(route = %uri, "request received for unknown URI");
    HandlerError::NotFound
}
