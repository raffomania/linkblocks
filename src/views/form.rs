use htmf::prelude::*;

use crate::form_errors::FormErrors;

pub fn errors(errors: &FormErrors, path: &str) -> Element {
  fragment().with(
    errors
      .filter(path)
      .iter()
      .map(|message| p(class("text-red-700")).with(message))
      .collect::<Vec<_>>(),
  )
}
