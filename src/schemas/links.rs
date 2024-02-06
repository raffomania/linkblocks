use garde::Validate;
use strum::EnumIter;
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug, EnumIter)]

pub enum ReferenceType {
    Bookmark,
    Note,
    List,
}

#[derive(Validate, Debug)]
pub struct CreateLink {
    #[garde(skip)]
    pub src_id: Uuid,
    #[garde(skip)]
    pub src_ref_type: ReferenceType,
    #[garde(skip)]
    pub dest_id: Uuid,
    #[garde(skip)]
    pub dest_ref_type: ReferenceType,
}
