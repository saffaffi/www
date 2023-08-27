use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use maud::{html, Markup};
use thiserror::Error;
use tracing::info;

use crate::templates::{pages, partials};

/// Errors that can be returned by request handlers.
#[derive(Error, Debug)]
pub enum HandlerError {
    /// The requested page was not found.
    #[error("page not found")]
    NotFound,

    /// An internal server error occurred while trying to handle the request.
    #[error("internal server error")]
    InternalError,
}

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(match self {
                HandlerError::NotFound => StatusCode::NOT_FOUND,
                HandlerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            })
            .body(match self {
                HandlerError::NotFound => body::boxed(pages::not_found().into_response()),
                HandlerError::InternalError => body::boxed(pages::internal_error().into_response()),
            })
            .unwrap()
    }
}

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
