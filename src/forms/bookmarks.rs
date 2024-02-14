use garde::Validate;
use serde::Deserialize;

#[derive(Validate, Default, Deserialize)]
pub struct CreateBookmark {
    #[garde(url)]
    pub url: String,
    #[garde(skip)]
    pub title: String,
}
