use maud::{html, Markup, PreEscaped, DOCTYPE};
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

use crate::state::ThemeSet;

pub fn head(theme_set: ThemeSet) -> Markup {
    let light_css = css_for_theme_with_class_style(
        theme_set.0.themes.get("OneHalfLight").unwrap(),
        ClassStyle::Spaced,
    )
    .unwrap();
    let light_block = format!(":root {{ {light_css} }}");

    let dark_css = css_for_theme_with_class_style(
        theme_set.0.themes.get("OneHalfDark").unwrap(),
        ClassStyle::Spaced,
    )
    .unwrap();
    let dark_block = format!("@media(prefers-color-scheme: dark) {{ :root{{ {dark_css} }} }}");

    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta viewport="width=device-width, initial-scale=1";
            link rel="stylesheet" href="/style.css" type="text/css";
            title { "saffi, wtf?!" }
            style {
                (PreEscaped(light_block))
                (PreEscaped(dark_block))
            }
        }
    }
}
