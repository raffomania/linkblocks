use super::layout;

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct Template {
    pub layout: layout::Template,
}
