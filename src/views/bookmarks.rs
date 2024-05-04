use askama::Template;

use crate::{
    db::{self},
    form_errors::FormErrors,
    forms,
};

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "create_bookmark.html")]
pub struct CreateBookmarkTemplate {
    pub layout: LayoutTemplate,

    pub errors: FormErrors,
    pub input: forms::bookmarks::CreateBookmark,
    pub selected_parents: Vec<db::List>,
    pub search_results: Vec<db::List>,
}

#[derive(Template)]
#[template(path = "unsorted_bookmarks.html")]
pub struct UnsortedBookmarksTemplate {
    pub layout: LayoutTemplate,
    pub bookmarks: Vec<db::Bookmark>,
}

#[derive(Template)]
#[template(path = "import_from_omnivore.html")]

pub struct ImportFromOmnivoreTemplate {
    pub layout: LayoutTemplate,
}
