use std::{collections::HashMap, fs, io, sync::Arc};

use axum::extract::FromRef;
use camino::Utf8PathBuf;
use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};
use syntect::{highlighting::ThemeSet as SyntectThemeSet, LoadingError as SyntectLoadingError};
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

        let theme_set = SyntectThemeSet::load_from_folder(self.themes_path)
            .map(Arc::new)
            .map(ThemeSet)?;

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

        Ok(State { content, theme_set })
    }
}

#[derive(Error, Debug)]
pub enum LoadStateError {
    #[error("failed to load theme set: {0}")]
    LoadThemeSet(#[from] SyntectLoadingError),

    #[error("failed to read page content: {0}")]
    ReadPageContent(#[source] io::Error),
}

#[derive(Clone, Debug)]
pub struct State {
    pub content: Content,
    pub theme_set: ThemeSet,
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
pub struct ThemeSet(pub Arc<SyntectThemeSet>);

impl FromRef<State> for ThemeSet {
    fn from_ref(input: &State) -> Self {
        input.theme_set.clone()
    }
}
