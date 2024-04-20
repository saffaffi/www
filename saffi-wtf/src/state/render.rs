use std::ops::Deref;

use maud::{html, Markup, PreEscaped, Render};
use tokio::sync::RwLockReadGuard;

use crate::state::{names::GroupName, Content, Page, Post};

// pub struct GroupRef<'a> {
//     pub guard: RwLockReadGuard<'a, Group>,
//     pub content: &'a Content,
// }
//
// impl<'a> Render for GroupRef<'a> {
//     fn render(&self) -> Markup {
//         let mb_index_content = self
//             .guard
//             .index
//             .as_ref()
//             .and_then(|page_name| self.content.pages.get(page_name))
//             .map(|page| page.html_content.as_str());
//
//         html! {
//             main class="page" {
//                 (if let Some(content) = mb_index_content {
//                     PreEscaped(content)
//                 } else {
//                     PreEscaped("")
//                 })
//             }
//         }
//     }
// }
//
// impl<'a> Deref for GroupRef<'a> {
//     type Target = Group;
//
//     fn deref(&self) -> &Self::Target {
//         self.guard.deref()
//     }
// }
//
// pub struct TagRef<'a> {
//     pub guard: RwLockReadGuard<'a, Tag>,
// }

pub struct PostRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, Post>,
}

impl<'a> Render for PostRef<'a> {
    fn render(&self) -> Markup {
        match self.guard.deref() {
            Post::Single {
                metadata,
                html_content,
            } => html! {
                article {
                    ul class="frontmatter" {
                        li { (metadata.date) }
                    }
                    (PreEscaped(&html_content))
                }
            },
            Post::Thread { metadata, entries } => html! {
                article {
                    p {
                        "threaded stuff coming later lol"
                    }
                }
            },
        }
    }
}

pub struct PageRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, Page>,
}

impl<'a> Render for PageRef<'a> {
    fn render(&self) -> Markup {
        let page = self.guard.deref();

        html! {
            (PreEscaped(&page.html_content))
        }
    }
}
