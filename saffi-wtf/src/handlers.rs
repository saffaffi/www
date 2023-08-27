use axum::{body::Body, extract::State, http::Request};
use maud::{html, Markup};
use tracing::info;

use crate::{errors::HandlerError, templates::partials, AppState};

pub async fn index(State(state): State<AppState>) -> Result<Markup, HandlerError> {
    info!(route = %"/", "handling request");
    Ok(html! {
        (partials::head(state).await)
        body {
            "Hello, wtf?!"
        }
    })
}

pub async fn make_green(State(state): State<AppState>) -> Result<(), HandlerError> {
    state.colours.write().await.error_background = "#cafeba";
    Ok(())
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
