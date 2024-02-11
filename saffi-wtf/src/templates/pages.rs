use std::fs;

use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};
use maud::{html, Markup, PreEscaped};

use crate::{templates::wrappers, AppState};

pub async fn index(state: AppState) -> Markup {
    let mut index_path = state.content_path.clone();
    index_path.push("_index.md");
    let raw_content = fs::read_to_string(index_path).unwrap();

    let syntect_adapter = SyntectAdapter::new(None);

    let plugins = {
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&syntect_adapter);
        plugins
    };
    let options = ComrakOptions::default();
    let html_content = markdown_to_html_with_plugins(&raw_content, &options, &plugins);

    wrappers::base(state, PreEscaped(html_content)).await
}

pub async fn not_found(state: AppState) -> Markup {
    wrappers::base(
        state,
        html! {
            main class="error" {
                h1 {
                    "not found"
                }

                p {
                    "wtf did you do?! that's not a route you can access."
                }
            }
        },
    )
    .await
}

pub async fn internal_error(state: AppState) -> Markup {
    wrappers::base(
        state,
        html! {
            main class="error" {
                h1 {
                    "internal server error"
                }

                p {
                    "wtf, you broke it?! stop doing that."
                }
            }
        },
    )
    .await
}
