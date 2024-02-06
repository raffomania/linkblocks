use garde::Validate;

#[derive(Validate)]
pub struct CreateNote {
    #[garde(skip)]
    pub content: String,
}
