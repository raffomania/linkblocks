use askama::Template;

use crate::{
    db::{self},
    form_errors::FormErrors,
    forms::lists::CreateList,
};

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
    pub layout: LayoutTemplate,
    pub links: Vec<db::LinkWithContent>,
    pub list: db::List,
}

#[derive(Template)]
#[template(path = "create_list.html")]
pub struct CreateListTemplate {
    pub layout: LayoutTemplate,
    pub input: CreateList,
    pub errors: FormErrors,
}
