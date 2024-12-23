#[allow(clippy::wildcard_imports)]
use htmf::prelude::*;

use crate::form_errors::FormErrors;

pub fn form_errors(errors: &FormErrors, path: &'static str) -> Element {
    errors
        .filter(path)
        .into_iter()
        .map(|e| p(class("text-red-700")).with(text(e)))
        .fold(fragment(), |frag, error_elem| frag.with(error_elem))
}
