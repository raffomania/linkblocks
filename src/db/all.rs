use sqlx::query;

use super::AppTx;
use crate::response_error::ResponseResult;

pub async fn wipe_all_data(tx: &mut AppTx) -> ResponseResult<()> {
    query!("truncate table links cascade;")
        .execute(&mut **tx)
        .await?;
    query!("truncate table lists cascade;")
        .execute(&mut **tx)
        .await?;
    query!("truncate table bookmarks cascade;")
        .execute(&mut **tx)
        .await?;
    query!("truncate table users cascade;")
        .execute(&mut **tx)
        .await?;

    Ok(())
}
