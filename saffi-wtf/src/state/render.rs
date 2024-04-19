use maud::{html, Markup, PreEscaped, Render};

use crate::state::{names::GroupName, Content, Group, Post, PostName, Tag};

pub struct GroupRef<'a> {
    pub group: &'a Group,
    pub content: &'a Content,
}

impl<'a> Render for GroupRef<'a> {
    fn render(&self) -> Markup {
        let mb_index_content = self
            .group
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

pub struct TagRef<'a> {
    pub tag: &'a Tag,
}

pub struct PostRef<'a> {
    pub post: &'a Post,
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
