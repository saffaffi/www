use std::fmt;

use maud::{html, Markup, Render};

pub struct DynamicColours {
    error_background: &'static str,
}

impl Default for DynamicColours {
    fn default() -> Self {
        Self {
            error_background: "#e56849",
        }
    }
}

impl fmt::Display for DynamicColours {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "--error-background: {};", self.error_background)?;

        Ok(())
    }
}

impl Render for DynamicColours {
    fn render(&self) -> Markup {
        html! {
            style {
                (format!(":root {{{self}}}"))
            }
        }
    }
}
