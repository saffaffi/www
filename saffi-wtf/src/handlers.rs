use axum::{body::Body, http::Request};
use maud::{html, Markup};
use tracing::info;

use crate::{errors::HandlerError, templates::partials};

pub async fn index() -> Result<Markup, HandlerError> {
    info!(route = %"/", "handling request");
    Ok(html! {
        (partials::head())
        body {
            "Hello, wtf?!"
        }
    })
}

pub async fn not_found(request: Request<Body>) -> HandlerError {
    let uri = request.uri();
    info!(route = %uri, "request received for unknown URI");
    HandlerError::NotFound
}

#[cfg(debug_assertions)]
pub async fn internal_error(request: Request<Body>) -> HandlerError {
    let uri = request.uri();
    info!(route = %uri, "internal error page explicitly requested");
    HandlerError::InternalError
}
