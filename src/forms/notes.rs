use garde::Validate;
use serde::Deserialize;

#[derive(Validate, Default, Deserialize)]
pub struct CreateNote {
    #[garde(length(min = 1, max = 100))]
    pub title: String,
    #[garde(skip)]
    pub content: Option<String>,
}
