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
    pub selected_parents: Vec<db::Note>,
    pub search_results: Vec<db::Note>,
}

#[derive(Template)]
#[template(path = "unlinked_bookmarks.html")]
pub struct UnlinkedBookmarksTemplate {
    pub layout: LayoutTemplate,
    pub bookmarks: Vec<db::Bookmark>,
}
