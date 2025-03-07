use htmf::{into_attrs::IntoAttrs, prelude::*};

use crate::{form_errors::FormErrors, forms::lists::CreateList};

use super::{form, layout};

pub struct Data {
  pub layout: layout::Template,
  pub input: CreateList,
  pub errors: FormErrors,
}

pub fn view(data: &Data) -> Element {
  layout::layout(
    fragment().with([form([
      action("/lists/create"),
      method("POST"),
      class("flex flex-col max-w-xl mx-4 mb-4 grow"),
    ])
    .with([
      header(class("mt-3 mb-4")).with([h1(class("text-xl font-bold")).with("Create a list")]),
      label(for_("title")).with("Title"),
      form::errors(&data.errors, "title"),
      input([
        required(""),
        name("title"),
        type_("text"),
        value(&data.input.title),
        class("rounded py-1.5 px-3 mt-2 bg-neutral-900"),
      ]),
      label(class("mt-4")).with([
        text("Note"),
        form::errors(&data.errors, "content"),
        textarea([
          name("content"),
          placeholder(""),
          value(data.input.content.as_deref().unwrap_or("")),
          class("rounded py-1.5 px-3 mt-2 bg-neutral-900 block w-full"),
        ]),
      ]),
      div(class("mt-3 mb-5")).with([label(()).with([
        input([
          type_("checkbox"),
          name("private"),
          value("true"),
          data.input.private.then(checked).into_attrs(),
        ]),
        text("Private"),
      ])]),
      form::errors(&data.errors, "root"),
      button([
        type_("submit"),
        class("bg-neutral-300 py-1.5 px-3 text-neutral-900 rounded mt-4 self-end"),
      ])
      .with("Add List"),
    ])]),
    &data.layout,
  )
}
