use crate::{
    authentication::AuthUser,
    db::{self, AppTx},
    response_error::ResponseResult,
};

pub struct LayoutTemplate {
    pub logged_in_username: String,
    pub lists: Vec<db::List>,
    pub authenticated: bool,
}

impl LayoutTemplate {
    pub async fn from_db(tx: &mut AppTx, auth_user: Option<AuthUser>) -> ResponseResult<Self> {
        match auth_user{
            Some(user) => {
                let logged_in_username = user.user.username.clone();
                let pinned_lists = db::lists::list_pinned_by_user(tx, user.user_id).await?;
                Ok(LayoutTemplate {
                    logged_in_username,
                    lists: pinned_lists,
                    authenticated: true,
                })
            },
            None => {
                let logged_in_username = "".to_string();
                let pinned_lists = vec![];
                Ok(LayoutTemplate {
                    logged_in_username,
                    lists: pinned_lists,
                    authenticated: false,
                })
            }
        }
    }
}
