use sqlx::query_as;
use sqlx::FromRow;
use sqlx::Postgres;
use sqlx::Transaction;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app_error::AppResult;
use crate::schemas::notes::CreateNote;

#[derive(FromRow, Debug)]
pub struct Note {
    pub id: Uuid,
    pub created_at: OffsetDateTime,
    pub user_id: Uuid,

    pub content: String,
}

pub async fn insert(
    // TODO create a type alias for this
    // TODO rename to tx everywhere
    db: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    // TODO rename to create_<entity> everywhere
    create: CreateNote,
) -> AppResult<Note> {
    let note = query_as!(
        Note,
        r#"
        insert into notes 
        (user_id, content) 
        values ($1, $2)
        returning *"#,
        user_id,
        create.content
    )
    .fetch_one(&mut **db)
    .await?;

    Ok(note)
}
