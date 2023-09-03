use axum::{
    body::Body,
    extract::State,
    http::{header, Request, Response},
};
use maud::Markup;
use tracing::info;

use crate::{errors::HandlerError, templates::pages, AppState};

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

pub async fn index(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    info!(route = %request.uri(), "handling request");
    Ok(pages::index(state).await)
}

pub async fn stylesheet(request: Request<Body>) -> Result<Response<String>, HandlerError> {
    info!(route = %request.uri(), "handling request");
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(STYLESHEET.to_owned())
        .map_err(|_| HandlerError::InternalError)
}

pub async fn make_green(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<(), HandlerError> {
    info!(route = %request.uri(), "making the error background green");
    state.colours.write().await.error_background = "#cafeba";
    Ok(())
}

pub async fn not_found(request: Request<Body>) -> HandlerError {
    info!(route = %request.uri(), "request received for unknown URI");
    HandlerError::NotFound
}

#[cfg(debug_assertions)]
pub async fn internal_error(request: Request<Body>) -> HandlerError {
    info!(route = %request.uri(), "internal error page explicitly requested");
    HandlerError::InternalError
}
