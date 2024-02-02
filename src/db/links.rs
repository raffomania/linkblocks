use sqlx::{query_as, FromRow, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    app_error::AppResult,
    schemas::links::{CreateLink, ReferenceType},
};

#[derive(FromRow)]
pub struct Link {
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

pub async fn insert(
    db: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    create: CreateLink,
) -> AppResult<Link> {
    let src_bookmark_id = (create.src_ref_type == ReferenceType::Bookmark).then_some(create.src_id);
    let src_note_id = (create.src_ref_type == ReferenceType::Note).then_some(create.src_id);
    let src_list_id = (create.src_ref_type == ReferenceType::List).then_some(create.src_id);

    let dest_bookmark_id =
        (create.dest_ref_type == ReferenceType::Bookmark).then_some(create.dest_id);
    let dest_note_id = (create.dest_ref_type == ReferenceType::Note).then_some(create.dest_id);
    let dest_list_id = (create.dest_ref_type == ReferenceType::List).then_some(create.dest_id);

    let list = query_as!(
        Link,
        r#"
        insert into links
        (
            user_id,
            src_bookmark_id,
            src_note_id,
            src_list_id,
            dest_bookmark_id,
            dest_note_id,
            dest_list_id
        )
        values ($1, $2, $3, $4, $5, $6, $7)
        returning *"#,
        user_id,
        src_bookmark_id,
        src_note_id,
        src_list_id,
        dest_bookmark_id,
        dest_note_id,
        dest_list_id
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(list)
}
