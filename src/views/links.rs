use askama::Template;

use crate::{
    db::{self, LinkDestination},
    form_errors::FormErrors,
    forms::links::PartialCreateLink,
};

use super::layout;

#[derive(Template)]
#[template(path = "create_link.html")]
pub struct CreateLinkTemplate {
    pub layout: layout::Template,

    pub errors: FormErrors,
    pub input: PartialCreateLink,
    pub search_results: Vec<LinkDestination>,
    pub src_from_db: Option<LinkDestination>,
    pub dest_from_db: Option<LinkDestination>,
}
