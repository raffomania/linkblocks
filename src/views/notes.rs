use askama::Template;

use crate::{
    db::{self},
    form_errors::FormErrors,
    forms::notes::CreateNote,
};

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "note.html")]
pub struct NoteTemplate {
    pub layout: LayoutTemplate,
    pub links: Vec<db::LinkWithContent>,
    pub note: db::Note,
}

#[derive(Template)]
#[template(path = "create_note.html")]
pub struct CreateNoteTemplate {
    pub layout: LayoutTemplate,
    pub input: CreateNote,
    pub errors: FormErrors,
}
