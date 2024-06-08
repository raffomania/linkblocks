use garde::Validate;
use serde::Deserialize;

#[derive(Validate, Default, Deserialize)]
pub struct CreateList {
    #[garde(length(min = 1, max = 100))]
    pub title: String,
    #[garde(skip)]
    pub content: Option<String>,
    #[garde(skip)]
    pub rich_view: Option<bool>,
}
