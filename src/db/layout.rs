use uuid::Uuid;

use super::AppTx;
use crate::{db, response_error::ResponseResult};

pub struct AuthedInfo {
    pub username: String,
    pub lists: Vec<db::List>,
    pub user_id: Uuid,
}

pub async fn by_user_id(tx: &mut AppTx, user_id: Uuid) -> ResponseResult<AuthedInfo> {
    let lists = db::lists::list_pinned_by_user(tx, user_id).await?;
    let username = sqlx::query!(
        "
    select ap_users.username from ap_users
    join users on users.ap_user_id = ap_users.id
    where users.id = $1
    ",
        user_id
    )
    .fetch_one(&mut **tx)
    .await?
    .username;

    Ok(AuthedInfo {
        username,
        lists,
        user_id,
    })
}
