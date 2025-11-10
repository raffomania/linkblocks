use uuid::Uuid;

use super::AppTx;
use crate::{db, response_error::ResponseResult};

pub struct AuthedInfo {
    pub username: String,
    pub lists: Vec<db::List>,
    pub ap_user_id: Uuid,
}

pub async fn by_ap_user_id(tx: &mut AppTx, ap_user_id: Uuid) -> ResponseResult<AuthedInfo> {
    let lists = db::lists::list_pinned_by_user(tx, ap_user_id).await?;
    let username = sqlx::query!(
        "
    select ap_users.username from ap_users
    where id = $1
    ",
        ap_user_id
    )
    .fetch_one(&mut **tx)
    .await?
    .username;

    Ok(AuthedInfo {
        username,
        lists,
        ap_user_id,
    })
}
