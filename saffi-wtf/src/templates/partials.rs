use maud::{html, Markup, DOCTYPE};

use crate::state::Theme;

pub async fn head(theme: Theme) -> Markup {
    let theme_header = theme.theme_header();
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta viewport="width=device-width, initial-scale=1";
            link rel="stylesheet" href="/style.css" type="text/css";
            title { "saffi, wtf?!" }
            style {
                (theme_header)
            }
        }
    }
}
