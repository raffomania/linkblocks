use askama::Template;

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub layout: LayoutTemplate,
    pub base_url: String,
}
