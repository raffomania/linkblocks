use askama::Template;

use crate::{
    db::{self, Bookmark, LinkDestination},
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
    pub selected_parent: Option<LinkDestination>,
}

#[derive(Template)]
#[template(path = "unlinked_bookmarks.html")]
pub struct UnlinkedBookmarksTemplate {
    pub layout: LayoutTemplate,
    pub bookmarks: Vec<Bookmark>
}
