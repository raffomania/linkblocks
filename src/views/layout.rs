use crate::{
    authentication::AuthUser,
    db::{self, layout::AuthedInfo, AppTx},
    response_error::ResponseResult,
};

pub struct Template {
    pub authed_info: Option<AuthedInfo>,
}

impl Template {
    pub async fn from_db(tx: &mut AppTx, auth_user: Option<&AuthUser>) -> ResponseResult<Self> {
        let auth_info = if let Some(auth_user) = auth_user {
            Some(db::layout::by_user_id(tx, auth_user.user_id).await?)
        } else {
            None
        };
        Ok(Template {
            authed_info: auth_info,
        })
    }
}
