use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{db::bookmarks::InsertBookmark, form_errors::FormErrors};

#[derive(Validate, Default, Deserialize, Clone)]
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

impl TryFrom<CreateBookmark> for InsertBookmark {
    type Error = FormErrors;

    fn try_from(value: CreateBookmark) -> Result<Self, Self::Error> {
        value.validate(&())?;

        if !value.submitted {
            return Err(FormErrors::default());
        }

        Ok(InsertBookmark {
            parent: value.parent,
            url: value.url,
            title: value.title,
        })
    }
}
