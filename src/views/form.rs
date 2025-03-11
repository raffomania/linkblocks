use htmf::prelude::*;

pub fn errors(errors: &[String]) -> Element {
    fragment().with(
        errors
            .iter()
            .map(|message| p(class("text-red-700")).with(message))
            .collect::<Vec<_>>(),
    )
}
