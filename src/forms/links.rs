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
    #[garde(length(max = 100))]
    pub search_term_src: Option<String>,
    #[garde(length(max = 100))]
    pub search_term_dest: Option<String>,
    #[garde(skip)]
    pub src: Option<Uuid>,
    #[garde(skip)]
    pub dest: Option<Uuid>,
    #[garde(skip)]
    #[serde(default)]
    pub submitted: bool,
}

#[derive(Debug, Deserialize)]
pub struct RemoveLink {
    pub id: Uuid,
}
