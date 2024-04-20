use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Request, Response},
};
use maud::Markup;
use tracing::{info, warn};

use crate::{
    errors::HandlerError,
    state::{names::GroupName, Content, Theme},
    templates::pages,
};

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

pub async fn index(
    State(content): State<Content>,
    State(theme): State<Theme>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    info!(route = %request.uri(), "handling request");

    if let Some(page) = content.page("_index").await {
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

    // if let Some(page) = group
    //     .try_into()
    //     .ok()
    //     .and_then(|group| content.group(&group))
    // {
    //     Ok(pages::group(page, theme).await)
    // } else {
    Err(not_found(request).await)
    // }
}

pub async fn tagged(
    State(content): State<Content>,
    State(theme): State<Theme>,
    Path(tag): Path<String>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    // info!(route = %request.uri(), "handling request");
    //
    // if let Some(page) = tag.try_into().ok().and_then(|tag| content.tag(&tag)) {
    //     Ok(pages::tagged(page, theme).await)
    // } else {
    Err(not_found(request).await)
    // }
}

pub async fn post(
    State(content): State<Content>,
    State(theme): State<Theme>,
    Path(post): Path<String>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    info!(route = %request.uri(), "handling request");

    // let group = group.try_into().ok();
    // let post = post.try_into().ok();
    //
    // if let Some(post) = group
    //     .zip(post)
    //     .and_then(|(group, post)| content.post(&group, &post))
    // {
    //     Ok(pages::post(post, theme).await)
    // } else {
    Err(not_found(request).await)
    // }
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
