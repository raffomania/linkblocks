use crate::{
    db::{self, AppTx},
    response_error::{ResponseError, ResponseResult},
};
use time;
use uuid::Uuid;

pub async fn create_api(
    tx: &mut AppTx,
    user_id: Uuid,
    permissions: &str,
) -> ResponseResult<db::users_api::UserApi> {
    let api_key = Uuid::new_v4();
    let valid_until = time::OffsetDateTime::now_utc() + time::Duration::days(30);

    let user_api = match db::users_api::by_user_id(tx, user_id).await {
        Ok(_) => db::users_api::renew(tx, user_id, api_key).await?,
        Err(_) => db::users_api::insert(tx, user_id, api_key, valid_until, permissions).await?,
    };

    Ok(user_api)
}

pub async fn verify_api(tx: &mut AppTx, user_id: &str, api_key: &str) -> ResponseResult<bool> {
    let user_api = db::users_api::by_api_key(tx, api_key).await?;

    if user_api.valid_until < time::OffsetDateTime::now_utc() {
        return Err(ResponseError::NotAuthenticated.into());
    }

    return Ok(user_api.user_id.to_string() == user_id);
}
