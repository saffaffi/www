use maud::{html, Markup, DOCTYPE};

use crate::AppState;

pub async fn head(state: AppState) -> Markup {
    let colours = state.colours.read().await;
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta viewport="width=device-width, initial-scale=1";
            link rel="stylesheet" href="/style.css" type="text/css";
            link href="https://fonts.googleapis.com/css2?family=Lora" rel="stylesheet";
            title { "saffi, wtf?!" }
            (*colours)
        }
    }
}
