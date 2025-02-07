use askama::Template;

use crate::{
  db::{self},
  form_errors::FormErrors,
  forms,
};

use super::layout;

#[derive(Template)]
#[template(path = "create_bookmark.html")]
pub struct CreateBookmarkTemplate {
  pub layout: layout::Template,

  pub errors: FormErrors,
  pub input: forms::bookmarks::CreateBookmark,
  pub selected_parents: Vec<db::List>,
  pub search_results: Vec<db::List>,
}

#[derive(Template)]
#[template(path = "unsorted_bookmarks.html")]
pub struct UnsortedBookmarksTemplate {
  pub layout: layout::Template,
  pub bookmarks: Vec<db::Bookmark>,
}
