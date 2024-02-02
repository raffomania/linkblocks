use garde::Validate;

#[derive(Validate)]
pub struct CreateList {
    #[garde(length(min = 1, max = 100))]
    pub title: String,
}
