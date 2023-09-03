use maud::{html, Markup};

use crate::{templates::wrappers, AppState};

pub async fn index(state: AppState) -> Markup {
    wrappers::base(
        state,
        html! {
            main {
                h1 { "an h1" }
                h2 { "an h2" }
                h3 { "an h3" }
                h4 { "an h4" }
                h5 { "an h5" }
                h6 { "an h6" }

                p {
                    "hello, wtf?!"
                }
            }
        },
    )
    .await
}

pub async fn not_found(state: AppState) -> Markup {
    wrappers::base(
        state,
        html! {
            main class="error" {
                h1 {
                    "not found"
                }

                p {
                    "wtf did you do?! that's not a route you can access."
                }
            }
        },
    )
    .await
}

pub async fn internal_error(state: AppState) -> Markup {
    wrappers::base(
        state,
        html! {
            main class="error" {
                h1 {
                    "internal server error"
                }

                p {
                    "wtf, you broke it?! stop doing that."
                }
            }
        },
    )
    .await
}
