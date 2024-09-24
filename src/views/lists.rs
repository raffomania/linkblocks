use askama::Template;

use crate::{
    db::{self},
    form_errors::FormErrors,
    forms::lists::CreateList,
};

use super::layout;

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
    pub layout: layout::Template,
    pub links: Vec<db::LinkWithContent>,
    pub list: db::List,
    pub metadata: db::lists::Metadata,
}

#[derive(Template)]
#[template(path = "create_list.html")]
pub struct CreateListTemplate {
    pub layout: layout::Template,
    pub input: CreateList,
    pub errors: FormErrors,
}

#[derive(Template)]
#[template(path = "list_unpinned_lists.html")]
pub struct UnpinnedListsTemplate {
    pub layout: layout::Template,
    pub lists: Vec<db::lists::UnpinnedList>,
}
