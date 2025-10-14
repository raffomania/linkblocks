use anyhow::Result;
use sqlx::{FromRow, PgTransaction, Row};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::{federation, forms::ap_users::CreateApUser};

#[derive(sqlx::FromRow)]
struct User {
    id: Uuid,
    username: String,
}

pub async fn migrate(tx: &mut PgTransaction<'_>, base_url: &Url) -> Result<()> {
    let users_without_ap_user =
        sqlx::query(r"select id, username from users where ap_user_id is null")
            .fetch_all(&mut **tx)
            .await?;

    // dropme
    for user in users_without_ap_user {
        new_ap_user(base_url, User::from_row(&user)?, tx).await?;
    }

    Ok(())
}

async fn new_ap_user(base_url: &Url, user: User, tx: &mut PgTransaction<'_>) -> Result<()> {
    let ap_keypair = federation::signing::generate_keypair()?;

    let username = user.username;
    let ap_id = base_url.join("/ap/user/")?.join(&username)?;
    let inbox_url = base_url.join("/ap/inbox")?;

    let create_user = CreateApUser {
        ap_id,
        username,
        inbox_url,
        public_key: ap_keypair.public_key,
        private_key: Some(ap_keypair.private_key),
        last_refreshed_at: OffsetDateTime::now_utc(),
        display_name: None,
        bio: None,
    };

    let id: Uuid = sqlx::query(
        r"
            insert into ap_users
            (
                ap_id,
                username,
                inbox_url,
                public_key,
                private_key,
                last_refreshed_at,
                display_name,
                bio
            )
            values ($1, $2, $3, $4, $5, $6, $7, $8)
            returning id
        ",
    )
    .bind(create_user.ap_id.to_string())
    .bind(create_user.username)
    .bind(create_user.inbox_url.to_string())
    .bind(create_user.public_key)
    .bind(create_user.private_key)
    .bind(create_user.last_refreshed_at)
    .bind(create_user.display_name)
    .bind(create_user.bio)
    .fetch_one(&mut **tx)
    .await?
    .get(0);

    sqlx::query(
        "
        update users
        set ap_user_id = $1
        where id = $2
    ",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
