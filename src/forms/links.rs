use garde::Validate;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Validate, Debug, Deserialize)]
pub struct CreateLink {
    #[garde(skip)]
    pub src: Uuid,
    #[garde(skip)]
    pub dest: Uuid,
}

#[derive(Validate, Debug, Deserialize, Default)]
pub struct PartialCreateLink {
    #[garde(skip)]
    pub search_term: Option<String>,
    #[garde(skip)]
    pub src: Option<Uuid>,
    #[garde(skip)]
    pub dest: Option<Uuid>,
    #[garde(skip)]
    #[serde(default)]
    pub submitted: bool,
}
