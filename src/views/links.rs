use askama::Template;

use crate::{
    db::{self, LinkDestination},
    form_errors::FormErrors,
    forms::links::PartialCreateLink,
};

use super::layout::LayoutTemplate;

#[derive(Template)]
#[template(path = "create_link.html")]
pub struct CreateLinkTemplate {
    pub layout: LayoutTemplate,

    pub errors: FormErrors,
    pub input: PartialCreateLink,
    pub search_results: Vec<LinkDestination>,
    pub selected_src: Option<LinkDestination>,
    pub selected_dest: Option<LinkDestination>,
}
