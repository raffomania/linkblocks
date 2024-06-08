use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Validate, Default, Deserialize, Clone, Debug)]
pub struct CreateBookmark {
    #[garde(skip)]
    #[serde(default)]
    pub parents: Vec<Uuid>,
    #[garde(skip)]
    #[serde(default)]
    pub create_parents: Vec<String>,
    #[garde(url)]
    pub url: String,
    #[garde(skip)]
    pub title: String,
    #[garde(length(max = 100))]
    pub list_search_term: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub submitted: bool,
}
