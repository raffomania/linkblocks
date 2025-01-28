use askama::Template;
use url::Url;

use super::layout;

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub layout: layout::Template,
    pub base_url: Url,
}
