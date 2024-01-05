use askama::Template;

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub layout: LayoutTemplate,
}
