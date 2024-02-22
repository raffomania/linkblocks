use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Validate, Default, Deserialize)]
pub struct CreateBookmark {
    #[garde(skip)]
    pub parent: Option<Uuid>,
    #[garde(url)]
    pub url: String,
    #[garde(skip)]
    pub title: String,
}
