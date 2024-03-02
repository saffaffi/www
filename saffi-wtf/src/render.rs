use chrono::NaiveDate;
use maud::{html, Markup, PreEscaped, Render};

pub struct RenderPost<'p> {
    content: &'p str,
    date: &'p NaiveDate,
}

impl<'p> RenderPost<'p> {
    pub fn new(content: &'p str, date: &'p NaiveDate) -> Self {
        Self { content, date }
    }
}

impl<'p> Render for RenderPost<'p> {
    fn render(&self) -> Markup {
        html! {
            article {
                ul class="frontmatter" {
                    li { (self.date) }
                }
                (PreEscaped(&self.content))
            }
        }
    }
}

pub struct RenderStatic<'p> {
    content: &'p str,
}

impl<'p> RenderStatic<'p> {
    pub fn new(content: &'p str) -> Self {
        Self { content }
    }
}

impl<'p> Render for RenderStatic<'p> {
    fn render(&self) -> Markup {
        html! {
            main class="page" {
                (PreEscaped(&self.content))
            }
        }
    }
}
