use axum::{
    body::Body,
    extract::State,
    http::{header, Request, Response},
};
use maud::Markup;
use tracing::{info, warn};

use crate::{
    errors::HandlerError,
    state::{Content, ThemeSet},
    templates::pages,
};

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

pub async fn index(
    State(content): State<Content>,
    State(theme_set): State<ThemeSet>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    info!(route = %request.uri(), "handling request");
    Ok(pages::index(content, theme_set).await)
}

pub async fn stylesheet(request: Request<Body>) -> Result<Response<String>, HandlerError> {
    info!(route = %request.uri(), "handling request");
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(STYLESHEET.to_owned())
        .map_err(|_| HandlerError::InternalError)
}

pub async fn not_found(request: Request<Body>) -> HandlerError {
    warn!(route = %request.uri(), "request received for unknown URI");
    HandlerError::NotFound
}

#[cfg(debug_assertions)]
pub async fn internal_error(request: Request<Body>) -> HandlerError {
    warn!(route = %request.uri(), "internal error page explicitly requested");
    HandlerError::InternalError
}
