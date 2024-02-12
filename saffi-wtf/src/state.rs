use std::{collections::HashMap, fs, io, sync::Arc};

use axum::extract::FromRef;
use camino::Utf8PathBuf;
use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};
use maud::{html, Markup, PreEscaped};
use syntect::{
    highlighting::ThemeSet as SyntectThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle},
    Error as SyntectError, LoadingError as SyntectLoadingError,
};
use thiserror::Error;

use crate::Args;

#[derive(Clone, Debug)]
pub struct Config {
    pub content_path: Utf8PathBuf,
    pub static_path: Utf8PathBuf,
    pub themes_path: Utf8PathBuf,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let Args {
            content_path,
            static_path,
            themes_path,
            ..
        } = args;
        Self {
            content_path,
            static_path,
            themes_path,
        }
    }
}

impl Config {
    pub fn load_state(self) -> Result<State, LoadStateError> {
        use LoadStateError::*;

        let syntect_adapter = SyntectAdapter::new(None);
        let plugins = {
            let mut plugins = ComrakPlugins::default();
            plugins.render.codefence_syntax_highlighter = Some(&syntect_adapter);
            plugins
        };
        let options = ComrakOptions::default();

        let mut index_path = self.content_path.clone();
        index_path.push("_index.md");
        let raw_content = fs::read_to_string(index_path).map_err(ReadPageContent)?;

        let html_content = markdown_to_html_with_plugins(&raw_content, &options, &plugins);

        let mut pages = HashMap::default();
        pages.insert("_index".into(), Page { html_content });

        let pages = Arc::new(pages);
        let content = Content { pages };

        let theme = SyntectThemeSet::load_from_folder(self.themes_path)?.try_into()?;

        Ok(State { content, theme })
    }
}

#[derive(Error, Debug)]
pub enum LoadStateError {
    #[error("failed to load theme set: {0}")]
    LoadThemeSet(#[from] SyntectLoadingError),

    #[error(transparent)]
    CreateThemeError(#[from] CreateThemeError),

    #[error("failed to read page content: {0}")]
    ReadPageContent(#[source] io::Error),
}

#[derive(Clone, Debug)]
pub struct State {
    pub content: Content,
    pub theme: Theme,
}

#[derive(Clone, Debug)]
pub struct Content {
    pub pages: Arc<HashMap<String, Page>>,
}

impl FromRef<State> for Content {
    fn from_ref(input: &State) -> Self {
        input.content.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Page {
    pub html_content: String,
}

#[derive(Clone, Debug)]
pub struct Theme {
    theme_header: Markup,
}

impl TryFrom<SyntectThemeSet> for Theme {
    type Error = CreateThemeError;

    fn try_from(theme_set: SyntectThemeSet) -> Result<Self, Self::Error> {
        use CreateThemeError::*;

        let light_css = css_for_theme_with_class_style(
            theme_set.themes.get("OneHalfLight").unwrap(),
            ClassStyle::Spaced,
        )
        .map_err(GenerateThemeCss)?;
        let light_block = format!(":root {{ {light_css} }}");

        let dark_css = css_for_theme_with_class_style(
            theme_set.themes.get("OneHalfDark").unwrap(),
            ClassStyle::Spaced,
        )
        .map_err(GenerateThemeCss)?;
        let dark_block = format!("@media(prefers-color-scheme: dark) {{ :root{{ {dark_css} }} }}");

        Ok(Self {
            theme_header: html! {
                (PreEscaped(light_block))
                (PreEscaped(dark_block))
            },
        })
    }
}

#[derive(Error, Debug)]
pub enum CreateThemeError {
    #[error("failed to generate CSS for theme: {0}")]
    GenerateThemeCss(#[source] SyntectError),
}

impl Theme {
    pub fn theme_header(&self) -> &Markup {
        &self.theme_header
    }
}

impl FromRef<State> for Theme {
    fn from_ref(input: &State) -> Self {
        input.theme.clone()
    }
}
