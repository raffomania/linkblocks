use askama::Template;


use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "api_key.html")]
pub struct ApiKeyTemplate {
    pub layout: LayoutTemplate,

    pub url: String,
}
