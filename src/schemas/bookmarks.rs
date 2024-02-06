use garde::Validate;

#[derive(Validate)]
pub struct CreateBookmark {
    #[garde(url)]
    pub url: String,
    #[garde(skip)]
    pub title: String,
}
