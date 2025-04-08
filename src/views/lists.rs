use askama::Template;

use super::layout;
use crate::db::{self};

#[derive(Template)]
#[template(path = "list_unpinned_lists.html")]
pub struct UnpinnedListsTemplate {
    pub layout: layout::Template,
    pub lists: Vec<db::lists::UnpinnedList>,
}
