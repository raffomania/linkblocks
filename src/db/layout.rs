use anyhow::Context;
use sqlx::query;
use uuid::Uuid;

use crate::db;
use crate::response_error::ResponseResult;

use super::AppTx;

pub struct AuthedInfo {
  pub user_description: String,
  pub lists: Vec<db::List>,
  pub user_id: Uuid,
}

pub async fn by_user_id(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<AuthedInfo> {
  let user_description = query!(
    r#"
        select coalesce(username, email) from users
        where id = $1
        "#,
    user_id
  )
  .fetch_one(&mut **tx)
  .await?
  .coalesce
  .context("User has no username or email")?;

  let lists = db::lists::list_pinned_by_user(tx, user_id).await?;

  Ok(AuthedInfo {
    user_description,
    lists,
    user_id,
  })
}
