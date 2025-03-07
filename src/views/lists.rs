use askama::Template;
use uuid::Uuid;

use crate::{
  db::{self},
  form_errors::FormErrors,
  forms::{self},
};

use super::layout;

#[derive(Template)]
#[template(path = "list.html")]
pub struct ListTemplate {
  pub layout: layout::Template,
  pub links: Vec<db::LinkWithContent>,
  pub list: db::List,
  pub metadata: db::lists::Metadata,
}

#[derive(Template)]
#[template(path = "edit_list_title.html")]
pub struct EditListTitleTemplate {
  pub layout: layout::Template,
  pub input: forms::lists::EditTitle,
  pub errors: FormErrors,
  pub list_id: Uuid,
}

#[derive(Template)]
#[template(path = "list_unpinned_lists.html")]
pub struct UnpinnedListsTemplate {
  pub layout: layout::Template,
  pub lists: Vec<db::lists::UnpinnedList>,
}
