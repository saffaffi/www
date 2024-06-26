use std::{
    collections::{HashMap, HashSet},
    io,
    path::PathBuf,
    sync::Arc,
};

use axum::extract::FromRef;
use camino::Utf8PathBuf;
use chrono::naive::NaiveDate;
use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use syntect::{
    highlighting::ThemeSet as SyntectThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle},
    Error as SyntectError, LoadingError as SyntectLoadingError,
};
use thiserror::Error;
use tokio::fs::{self, DirEntry};
use tracing::info;

use crate::{
    state::{
        names::{GroupName, PageName, ParseGroupNameError, ParsePageNameError, TagName},
        render::{GroupRef, PostRef, TagRef},
    },
    Args,
};

pub mod names;
pub mod render;

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
        let mut pages = PagesMap::new();
        let mut posts = PostsMap::new();
        let mut groups_to_load = Vec::new();

        let load_page = |entry: DirEntry,
                         group_context: GroupName,
                         mut groups: GroupsMap,
                         mut tags: TagsMap,
                         mut pages: PagesMap,
                         mut posts: PostsMap| async move {
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

            if let Ok((date, _)) = NaiveDate::parse_and_remainder(&file_name, "%Y-%m-%d") {
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

                if frontmatter.draft && !self.drafts {
                    info!(?path, "skipping draft");
                } else {
                    let html_content = markdown_to_html(raw_markdown);

                    posts.insert(
                        page_name,
                        Post {
                            date,
                            frontmatter,
                            html_content,
                        },
                    );

                    info!(?path, "loaded post");
                }
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

                let html_content = markdown_to_html(raw_markdown);

                pages.insert(page_name, Page { html_content });

                info!(?path, "loaded static page");
            };

            Ok::<_, LoadStateError>((groups, tags, pages, posts))
        };

        let mut top_level_reader = fs::read_dir(&self.content_path).await.map_err(ReadDir)?;
        while let Some(entry) = top_level_reader.next_entry().await.map_err(ReadDirEntry)? {
            if entry.metadata().await.map_err(DirEntryMetadata)?.is_file() {
                (groups, tags, pages, posts) =
                    load_page(entry, GroupName::Root, groups, tags, pages, posts).await?;
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
                    (groups, tags, pages, posts) =
                        load_page(entry, group.clone(), groups, tags, pages, posts).await?;
                } else {
                    info!(path = ?entry.path(), "skipping nested group");
                }
            }
        }

        let groups = Arc::new(groups);
        let tags = Arc::new(tags);
        let pages = Arc::new(pages);
        let posts = Arc::new(posts);
        let content = Content {
            groups,
            tags,
            pages,
            posts,
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

type PostName = PageName;

type GroupsMap = HashMap<GroupName, Group>;
type TagsMap = HashMap<TagName, Tag>;
type PagesMap = HashMap<PageName, Page>;
type PostsMap = HashMap<PostName, Post>;

#[derive(Clone, Debug)]
pub struct Content {
    groups: Arc<GroupsMap>,
    tags: Arc<TagsMap>,
    pages: Arc<PagesMap>,
    posts: Arc<PostsMap>,
}

impl Content {
    pub fn group(&self, group_name: &GroupName) -> Option<GroupRef<'_>> {
        self.groups.get(group_name).map(|group| GroupRef {
            group,
            content: self,
        })
    }

    pub fn tag(&self, tag_name: &TagName) -> Option<TagRef<'_>> {
        self.tags.get(tag_name).map(|tag| TagRef { tag })
    }

    pub fn post(&self, group_name: &GroupName, post_name: &PostName) -> Option<PostRef<'_>> {
        let post_name = self
            .groups
            .get(group_name)
            .and_then(|group| group.members.get(post_name))?;
        self.posts.get(post_name).map(|post| PostRef {
            post,
            group_name: group_name.clone(),
            name: post_name,
            content: self,
        })
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
pub struct Post {
    pub date: NaiveDate,
    pub frontmatter: PostFrontmatter,
    pub html_content: String,
}

#[derive(Clone, Debug)]
pub struct Page {
    pub html_content: String,
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
