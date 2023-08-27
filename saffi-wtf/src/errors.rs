use axum::{
    body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::templates::pages;

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
