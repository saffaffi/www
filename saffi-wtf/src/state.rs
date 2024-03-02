use std::{
    collections::{HashMap, HashSet},
    fmt, io,
    path::PathBuf,
    sync::Arc,
};

use axum::extract::FromRef;
use camino::Utf8PathBuf;
use chrono::naive::NaiveDate;
use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};
use maud::{html, Markup, PreEscaped, Render};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use syntect::{
    highlighting::ThemeSet as SyntectThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle},
    Error as SyntectError, LoadingError as SyntectLoadingError,
};
use thiserror::Error;
use tokio::fs::{self, DirEntry};
use tracing::info;
use uuid::Uuid;
use www_saffi::SwapResult;

use crate::{
    render::{RenderPost, RenderStatic},
    Args,
};

#[derive(Clone, Debug)]
pub struct Config {
    pub drafts: bool,
    pub content_path: Utf8PathBuf,
    pub static_path: Utf8PathBuf,
    pub themes_path: Utf8PathBuf,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let Args {
            drafts,
            content_path,
            static_path,
            themes_path,
            ..
        } = args;
        Self {
            drafts,
            content_path,
            static_path,
            themes_path,
        }
    }
}

impl Config {
    pub async fn load_state(self) -> Result<State, LoadStateError> {
        use LoadStateError::*;

        let theme_set = SyntectThemeSet::load_from_folder(self.themes_path)?;
        let theme = Theme::try_load(theme_set, "OneHalfLight", "OneHalfDark")?;

        let syntect_adapter = SyntectAdapter::new(None);
        let plugins = {
            let mut plugins = ComrakPlugins::default();
            plugins.render.codefence_syntax_highlighter = Some(&syntect_adapter);
            plugins
        };
        let options = ComrakOptions::default();

        let markdown_to_html = |md: &str| markdown_to_html_with_plugins(md, &options, &plugins);

        let mut groups = GroupsMap::new();
        let mut tags = TagsMap::new();
        let mut pages = HashMap::new();
        let mut groups_to_load = Vec::new();

        let load_page = |entry: DirEntry,
                         group_context: GroupName,
                         mut groups: GroupsMap,
                         mut tags: TagsMap,
                         mut pages: PagesMap| async move {
            let path = entry.path();
            let file_name = path
                .file_stem()
                .ok_or_else(|| NoFileStem(entry.path()))?
                .to_str()
                .ok_or_else(|| PathInvalidUtf8(entry.path()))?
                .to_owned();

            let file_ext = path
                .extension()
                .ok_or_else(|| NoFileExt(entry.path()))?
                .to_str()
                .ok_or_else(|| PathInvalidUtf8(entry.path()))?;

            if file_ext != "md" && file_ext != "markdown" {
                panic!("found a file that's not markdown: {path:?}");
            }

            let page_name: PageName = if file_name == "_index" {
                let name = PageName::new_index();
                groups.entry(group_context).or_default().index = Some(name.clone());
                name
            } else {
                let name: PageName = file_name.clone().try_into()?;
                groups
                    .entry(group_context)
                    .or_default()
                    .members
                    .insert(name.clone());
                name
            };

            let raw_content = fs::read_to_string(&path).await.map_err(ReadPageContent)?;

            let (page_type, raw_markdown) = if let Ok((date, _)) =
                NaiveDate::parse_and_remainder(&file_name, "%Y-%m-%d")
            {
                let (raw_frontmatter, raw_markdown) = raw_content
                    .strip_prefix("---")
                    .ok_or_else(|| MissingFrontmatter(entry.path()))?
                    .trim()
                    .split_once("---")
                    .ok_or_else(|| MalformedFrontmatter(entry.path()))?;

                let frontmatter = toml::from_str::<PostFrontmatter>(raw_frontmatter)?;

                for tag in frontmatter.tags.iter().cloned() {
                    tags.entry(tag)
                        .or_default()
                        .members
                        .insert(page_name.clone());
                }

                (PageType::Post { date, frontmatter }, raw_markdown)
            } else {
                let raw_markdown = if let Some(stripped_once) = raw_content.strip_prefix("---") {
                    // For now, we're ignoring any frontmatter. Later, when static pages need
                    // frontmatter, we'll read this.
                    let (_, raw_markdown) = stripped_once
                        .trim()
                        .split_once("---")
                        .ok_or_else(|| MalformedFrontmatter(entry.path()))?;
                    raw_markdown
                } else {
                    raw_content.as_str()
                };

                (PageType::Static, raw_markdown)
            };

            if page_type.is_draft() && !self.drafts {
                info!(?path, "skipping draft");
            } else {
                info!(?path, "loaded page");

                let html_content = markdown_to_html(raw_markdown);
                pages.insert(
                    page_name,
                    Page {
                        page_type,
                        html_content,
                    },
                );
            }

            Ok::<_, LoadStateError>((groups, tags, pages))
        };

        let mut top_level_reader = fs::read_dir(&self.content_path).await.map_err(ReadDir)?;
        while let Some(entry) = top_level_reader.next_entry().await.map_err(ReadDirEntry)? {
            if entry.metadata().await.map_err(DirEntryMetadata)?.is_file() {
                (groups, tags, pages) =
                    load_page(entry, GroupName::Root, groups, tags, pages).await?;
            } else {
                let group_name = entry
                    .file_name()
                    .to_str()
                    .ok_or_else(|| PathInvalidUtf8(entry.path()))?
                    .to_string();
                let group: GroupName = group_name.try_into()?;
                groups.insert(group.clone(), <_>::default());
                groups_to_load.push((entry.path(), group));
            }
        }

        for (group_path, group) in groups_to_load {
            let mut group_reader = fs::read_dir(group_path).await.map_err(ReadDir)?;

            while let Some(entry) = group_reader.next_entry().await.map_err(ReadDirEntry)? {
                if entry.metadata().await.map_err(DirEntryMetadata)?.is_file() {
                    (groups, tags, pages) =
                        load_page(entry, group.clone(), groups, tags, pages).await?;
                } else {
                    info!(path = ?entry.path(), "skipping nested group");
                }
            }
        }

        let groups = Arc::new(dbg!(groups));
        let tags = Arc::new(dbg!(tags));
        let pages = Arc::new(dbg!(pages));
        let content = Content {
            groups,
            tags,
            pages,
        };

        Ok(State { content, theme })
    }
}

#[derive(Error, Debug)]
pub enum LoadStateError {
    #[error("failed to load theme set: {0}")]
    LoadThemeSet(#[from] SyntectLoadingError),

    #[error(transparent)]
    LoadThemeError(#[from] LoadThemeError),

    #[error("failed to read contents of dir: {0}")]
    ReadDir(#[source] io::Error),

    #[error("failed to read dir entry: {0}")]
    ReadDirEntry(#[source] io::Error),

    #[error("failed to access metadata of dir entry: {0}")]
    DirEntryMetadata(#[source] io::Error),

    #[error("file path does not contain a file stem: {0}")]
    NoFileStem(PathBuf),

    #[error("file path does not contain an extension: {0}")]
    NoFileExt(PathBuf),

    #[error("invalid UTF-8 in file path: {0}")]
    PathInvalidUtf8(PathBuf),

    #[error(transparent)]
    ParseGroupError(#[from] ParseGroupNameError),

    #[error(transparent)]
    ParsePageNameError(#[from] ParsePageNameError),

    #[error("failed to read page content: {0}")]
    ReadPageContent(#[source] io::Error),

    #[error("page at path {0} does not begin with frontmatter")]
    MissingFrontmatter(PathBuf),

    #[error("frontmatter of page at path {0} is malformed")]
    MalformedFrontmatter(PathBuf),

    #[error("failed to parse page frontmatter: {0}")]
    ParseFrontmatter(#[from] toml::de::Error),
}

#[derive(Clone, Debug)]
pub struct State {
    pub content: Content,
    pub theme: Theme,
}

/// The name of a group, either parsed from a raw string or the root group
/// (which has no name).
///
/// Group names are single path components in a URL, containing only lowercase
/// ASCII-alphabetic characters and dashes.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum GroupName {
    Root,
    Named(String),
}

impl TryFrom<String> for GroupName {
    type Error = ParseGroupNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        use ParseGroupNameError::*;

        // Look for any characters that are not lowercase ASCII-alphabetic or
        // dashes. If any are found, this is an invalid group name, and the
        // invalid char will be returned in Some().
        raw.chars()
            .find(|&c| !(c.is_ascii_lowercase() || c == '-'))
            .map(|inv| InvalidChar(raw.clone(), inv))
            .ok_or(GroupName::Named(raw))
            .swap()
    }
}

#[derive(Error, Debug)]
pub enum ParseGroupNameError {
    #[error("group name \"{0}\" contains invalid char '{1}'")]
    InvalidChar(String, char),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TagName(String);

impl TryFrom<String> for TagName {
    type Error = ParseTagNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        use ParseTagNameError::*;

        // Look for any characters that are not lowercase ASCII-alphabetic or
        // dashes. If any are found, this is an invalid group name, and the
        // invalid char will be returned in Some().
        raw.chars()
            .find(|&c| !(c.is_ascii_lowercase() || c == '-'))
            .map(|inv| InvalidChar(raw.clone(), inv))
            .ok_or(TagName(raw))
            .swap()
    }
}

impl TryFrom<&str> for TagName {
    type Error = ParseTagNameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::try_from(raw.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum ParseTagNameError {
    #[error("tag name \"{0}\" contains invalid char '{1}'")]
    InvalidChar(String, char),
}

struct TagNameVisitor;

impl<'de> Visitor<'de> for TagNameVisitor {
    type Value = TagName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a string containing only lowercase ASCII-alphabetic characters or dashes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        TagName::try_from(v).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TagNameVisitor)
    }
}

/// The name of a page, either parsed from a raw string or an index page, which
/// has no name beyond the name of the group it's contained in.
///
/// Page names are single path components in a URL, containing only numerals,
/// lowercase ASCII-alphabetic characters and dashes.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PageName {
    Index(Uuid),
    Named(String),
}

impl PageName {
    pub fn new_index() -> Self {
        Self::Index(Uuid::new_v4())
    }
}

impl TryFrom<String> for PageName {
    type Error = ParsePageNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        use ParsePageNameError::*;

        // Look for any characters that are not lowercase ASCII-alphabetic or
        // dashes. If any are found, this is an invalid group name, and the
        // invalid char will be returned in Some().
        raw.chars()
            .find(|&c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
            .map(|inv| InvalidChar(raw.clone(), inv))
            .ok_or(PageName::Named(raw))
            .swap()
    }
}

impl TryFrom<&str> for PageName {
    type Error = ParsePageNameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::try_from(raw.to_owned())
    }
}

#[derive(Error, Debug)]
pub enum ParsePageNameError {
    #[error("page name \"{0}\" contains invalid char '{1}'")]
    InvalidChar(String, char),
}

type GroupsMap = HashMap<GroupName, Group>;
type TagsMap = HashMap<TagName, Tag>;
type PagesMap = HashMap<PageName, Page>;

#[derive(Clone, Debug)]
pub struct Content {
    groups: Arc<GroupsMap>,
    tags: Arc<TagsMap>,
    pages: Arc<PagesMap>,
}

impl Content {
    pub fn index(&self, group_name: &GroupName) -> Option<&Page> {
        let page_name = self
            .groups
            .get(group_name)
            .and_then(|group| group.index.as_ref())?;
        self.pages.get(page_name)
    }

    pub fn page(&self, group_name: &GroupName, page_name: &PageName) -> Option<&Page> {
        let page_name = self
            .groups
            .get(group_name)
            .and_then(|group| group.members.get(page_name))?;
        self.pages.get(page_name)
    }
}

impl FromRef<State> for Content {
    fn from_ref(input: &State) -> Self {
        input.content.clone()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Group {
    index: Option<PageName>,
    members: HashSet<PageName>,
}

#[derive(Clone, Debug, Default)]
pub struct Tag {
    members: HashSet<PageName>,
}

#[derive(Clone, Debug)]
pub struct Page {
    page_type: PageType,
    html_content: String,
}

impl Render for Page {
    fn render(&self) -> Markup {
        match self.page_type {
            PageType::Post { ref date, .. } => RenderPost::new(&self.html_content, date).render(),
            PageType::Static => RenderStatic::new(&self.html_content).render(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PageType {
    Post {
        date: NaiveDate,
        frontmatter: PostFrontmatter,
    },
    Static,
}

impl PageType {
    fn is_draft(&self) -> bool {
        match self {
            PageType::Post { frontmatter, .. } => frontmatter.draft,
            PageType::Static => false,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostFrontmatter {
    #[serde(default)]
    draft: bool,

    #[serde(default)]
    tags: Vec<TagName>,
}

#[derive(Clone, Debug)]
pub struct Theme {
    theme_header: Markup,
}

impl Theme {
    pub fn try_load(
        theme_set: SyntectThemeSet,
        light: &'static str,
        dark: &'static str,
    ) -> Result<Self, LoadThemeError> {
        use LoadThemeError::*;

        let light_css = css_for_theme_with_class_style(
            theme_set
                .themes
                .get(light)
                .ok_or_else(|| MissingTheme(light))?,
            ClassStyle::Spaced,
        )
        .map_err(GenerateThemeCss)?;
        let light_block = format!(":root {{ {light_css} }}");

        let dark_css = css_for_theme_with_class_style(
            theme_set
                .themes
                .get(dark)
                .ok_or_else(|| MissingTheme(dark))?,
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
pub enum LoadThemeError {
    #[error("failed to generate CSS for theme: {0}")]
    GenerateThemeCss(#[source] SyntectError),

    #[error("theme set does not contain a theme with name: {0}")]
    MissingTheme(&'static str),
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
