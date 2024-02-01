use garde::Validate;
use uuid::Uuid;

#[derive(Validate)]
pub struct CreateBookmark {
    #[garde(skip)]
    pub user_id: Uuid,
    #[garde(url)]
    pub url: String,
}
