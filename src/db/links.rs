use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(FromRow)]
pub struct InternalLink {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub src_bookmark_id: Option<Uuid>,
    pub src_note_id: Option<Uuid>,
    pub src_list_id: Option<Uuid>,

    pub dest_bookmark_id: Option<Uuid>,
    pub dest_note_id: Option<Uuid>,
    pub dest_list_id: Option<Uuid>,
}
