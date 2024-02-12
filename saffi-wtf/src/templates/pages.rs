use maud::{html, Markup, PreEscaped};

use crate::{
    state::{Content, Theme},
    templates::wrappers,
};

pub async fn index(content: Content, theme: Theme) -> Markup {
    let page = content.pages.get("_index").unwrap();
    wrappers::base(
        theme,
        html! {
            (PreEscaped(&page.html_content))
        },
    )
    .await
}

pub async fn not_found(theme: Theme) -> Markup {
    wrappers::base(
        theme,
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

pub async fn internal_error(theme: Theme) -> Markup {
    wrappers::base(
        theme,
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
