use std::ops::Deref;

use maud::{html, Markup, PreEscaped, Render};
use tokio::sync::RwLockReadGuard;

use crate::state::{names::GroupName, Content, Group, Page, Post, PostName, Tag};

pub struct GroupRef<'a> {
    pub guard: RwLockReadGuard<'a, Group>,
    pub content: &'a Content,
}

impl<'a> Render for GroupRef<'a> {
    fn render(&self) -> Markup {
        let mb_index_content = self
            .guard
            .index
            .as_ref()
            .and_then(|page_name| self.content.pages.get(page_name))
            .map(|page| page.html_content.as_str());

        html! {
            main class="page" {
                (if let Some(content) = mb_index_content {
                    PreEscaped(content)
                } else {
                    PreEscaped("")
                })
            }
        }
    }
}

impl<'a> Deref for GroupRef<'a> {
    type Target = Group;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct TagRef<'a> {
    pub guard: RwLockReadGuard<'a, Tag>,
}

pub struct PostRef<'a> {
    pub guard: RwLockReadGuard<'a, Post>,
    pub group_name: GroupName,
    pub name: &'a PostName,
    pub content: &'a Content,
}

impl<'a> Render for PostRef<'a> {
    fn render(&self) -> Markup {
        html! {
            article {
                ul class="frontmatter" {
                    li { (self.post.date) }
                }
                (PreEscaped(&self.post.html_content))
            }
        }
    }
}

pub struct PageRef<'a> {
    guard: RwLockReadGuard<'a, Page>,
    pub content: &'a Content,
}
