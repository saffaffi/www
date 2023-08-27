use maud::{html, Markup};

use super::partials;
use crate::AppState;

pub async fn not_found(state: AppState) -> Markup {
    html! {
        (partials::head(state).await)
        body {
            main class="error" {
                h1 {
                    "not found"
                }

                p {
                    "wtf did you do?! that's not a route you can access."
                }
            }
        }
    }
}

pub async fn internal_error(state: AppState) -> Markup {
    html! {
        (partials::head(state).await)
        body {
            main class="error" {
                h1 {
                    "internal server error"
                }

                p {
                    "wtf, you broke it?! stop doing that."
                }
            }
        }
    }
}
