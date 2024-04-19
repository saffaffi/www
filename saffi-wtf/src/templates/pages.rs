use maud::{html, Markup};

use crate::{
    state::{
        render::{GroupRef, PostRef, TagRef},
        Theme,
    },
    templates::wrappers,
};

pub async fn post(page: PostRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        theme,
        html! {
            (page)
        },
    )
    .await
}

pub async fn group(group: GroupRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        theme,
        html! {
            (group)
        },
    )
    .await
}

pub async fn tagged(_tag: TagRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        theme,
        html! {
            ("nothing here yet")
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
