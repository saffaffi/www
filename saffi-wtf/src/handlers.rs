use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Request, Response},
};
use maud::Markup;
use tracing::{info, warn};

use crate::{
    errors::HandlerError,
    state::{Content, GroupName, Theme},
    templates::pages,
};

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

pub async fn index(
    State(content): State<Content>,
    State(theme): State<Theme>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    info!(route = %request.uri(), "handling request");

    if let Some(page) = content.index(&GroupName::Root) {
        Ok(pages::page(page, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn group(
    State(content): State<Content>,
    State(theme): State<Theme>,
    Path(group): Path<String>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    info!(route = %request.uri(), "handling request");

    if let Some(page) = group
        .try_into()
        .ok()
        .and_then(|group| content.index(&group))
    {
        Ok(pages::page(page, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn page(
    State(content): State<Content>,
    State(theme): State<Theme>,
    Path((group, page)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    info!(route = %request.uri(), "handling request");

    let group = group.try_into().ok();
    let page = page.try_into().ok();

    if let Some(page) = group
        .zip(page)
        .and_then(|(group, page)| content.page(&group, &page))
    {
        Ok(pages::page(page, theme).await)
    } else {
        Err(not_found(request).await)
    }
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
