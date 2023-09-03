use maud::{html, Markup};

use crate::{templates::partials, AppState};

pub async fn base(state: AppState, content: Markup) -> Markup {
    html! {
        (partials::head(state).await)
        body {
            h1 class="sitetitle" {
                a href="/" {
                    "saffi, wtf?!"
                }
            }

            (content)
        }
    }
}
