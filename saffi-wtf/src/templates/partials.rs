use maud::{html, Markup, DOCTYPE};

use crate::templates::components::DynamicColours;

pub fn head() -> Markup {
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta viewport="width=device-width, initial-scale=1";
            title { "saffi, wtf?!" }
            (DynamicColours::default())
        }
    }
}
