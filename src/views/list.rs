use askama::Template;

use crate::db::{self};

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
    pub layout: LayoutTemplate,
    pub links: Vec<db::LinkWithContent>,
    pub list: db::List,
}
