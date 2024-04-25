use crate::{
    authentication::AuthUser,
    db::{self, AppTx},
    response_error::ResponseResult,
};

pub struct Template {
    pub logged_in_username: String,
    pub lists: Vec<db::List>,
}

impl Template {
    pub async fn from_db(tx: &mut AppTx, auth_user: &AuthUser) -> ResponseResult<Self> {
        let pinned_lists = db::lists::list_pinned_by_user(tx, auth_user.user_id).await?;
        let logged_in_username = auth_user
            .user
            .username
            .as_ref()
            .or(auth_user.user.email.as_ref())
            .unwrap_or(&String::new())
            .clone();
        Ok(Template {
            logged_in_username,
            lists: pinned_lists,
        })
    }
}
