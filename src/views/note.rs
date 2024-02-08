use askama::Template;

use crate::db::{self};

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "note.html")]
pub struct NoteTemplate {
    pub layout: LayoutTemplate,
    pub links: Vec<db::LinkWithContent>,
    pub note: db::Note,
}
