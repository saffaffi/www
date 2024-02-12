use maud::{html, Markup};

use crate::{state::Theme, templates::partials};

pub async fn base(theme: Theme, content: Markup) -> Markup {
    html! {
        (partials::head(theme).await)
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
