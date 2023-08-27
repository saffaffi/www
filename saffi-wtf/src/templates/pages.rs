use maud::{html, Markup};

use super::partials;
use crate::AppState;

pub async fn not_found(state: AppState) -> Markup {
    html! {
        (partials::head(state).await)
        body {
            p style="background: var(--error-background, #ffffff)" {
                "wtf did you do?! that's not a route you can access."
            }
        }
    }
}

pub async fn internal_error(state: AppState) -> Markup {
    html! {
        (partials::head(state).await)
        body {
            p style="background: var(--error-background, #ffffff)" {
                "wtf, you broke it?! stop doing that."
            }
        }
    }
}
