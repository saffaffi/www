use maud::{html, Markup};

use super::partials;

pub fn not_found() -> Markup {
    html! {
        (partials::head())
        body {
            p {
                "wtf did you do?! that's not a route you can access."
            }
        }
    }
}

pub fn internal_error() -> Markup {
    html! {
        (partials::head())
        body {
            p {
                "wtf, you broke it?! stop doing that."
            }
        }
    }
}
