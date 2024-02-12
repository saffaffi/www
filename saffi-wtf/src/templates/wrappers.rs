use maud::{html, Markup, DOCTYPE};

use crate::{state::Theme, templates::partials};

pub async fn base(theme: Theme, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en-GB" {
            (partials::head(theme).await)
            body {
                header {
                    h1 class="sitetitle" {
                        a href="/" {
                            "saffi, wtf?!"
                        }
                    }
                }

                (content)
            }
        }
    }
}
