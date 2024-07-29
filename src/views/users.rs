use askama::Template;

use super::layout;

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub layout: layout::Template,
    pub base_url: String,
}
