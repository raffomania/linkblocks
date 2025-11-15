use sqlx::{prelude::FromRow, query_as};
use uuid::Uuid;

use crate::{db::AppTx, response_error::ResponseResult};

#[expect(dead_code)]
#[derive(FromRow, Debug)]
pub struct Follow {
    pub id: Uuid,

    pub follower_id: Uuid,
    pub following_id: Uuid,
}

pub struct Insert {
    pub follower_id: Uuid,
    pub following_id: Uuid,
}

pub async fn upsert(tx: &mut AppTx, insert: Insert) -> ResponseResult<Follow> {
    let follow = query_as!(
        Follow,
        r"
        insert into follows
        (
            follower_id,
            following_id
        )
        values ($1, $2)
        on conflict (follower_id, following_id)
            do nothing
        returning *
        ",
        insert.follower_id,
        insert.following_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(follow)
}
