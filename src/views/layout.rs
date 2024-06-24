use crate::{
    authentication::AuthUser,
    db::{self, AppTx},
    response_error::ResponseResult,
};

pub struct LayoutTemplate {
    pub logged_in_username: String,
    pub lists: Vec<db::List>,
}

impl LayoutTemplate {
    pub async fn from_db(tx: &mut AppTx, auth_user: &AuthUser) -> ResponseResult<Self> {
        let pinned_lists = db::lists::list_pinned_by_user(tx, auth_user.user_id).await?;
        Ok(LayoutTemplate {
            logged_in_username: auth_user.user.username.clone(),
            lists: pinned_lists,
        })
    }
}
