use maud::{html, Markup};

use crate::{state::ThemeSet, templates::partials};

pub async fn base(theme_set: ThemeSet, content: Markup) -> Markup {
    html! {
        (partials::head(theme_set))
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
