use maud::{html, Markup};

use crate::state::Theme;

pub async fn head(theme: Theme) -> Markup {
    let theme_header = theme.theme_header();
    html! {
        head {
            meta charset="utf-8";
            meta viewport="width=device-width, initial-scale=1";

            link rel="preload" href="/static/iosevka-regular.woff2" as="font" type="font/woff2" crossorigin;
            link rel="preload" href="/static/lora-regular.woff2" as="font" type="font/woff2" crossorigin;
            link rel="preload" href="/static/lora-italic.woff2" as="font" type="font/woff2" crossorigin;
            link rel="preload" href="/static/lora-600.woff2" as="font" type="font/woff2" crossorigin;

            link rel="stylesheet" href="/style.css" type="text/css";

            title { "saffi, wtf?!" }
            style {
                (theme_header)
            }
        }
    }
}
