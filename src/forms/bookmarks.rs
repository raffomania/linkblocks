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
    #[garde(length(max = 100))]
    pub note_search_term: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub submitted: bool,
}
