use sqlx::Row;
use url::Url;
use uuid::Uuid;

use crate::db;

#[test_log::test(tokio::test)]
async fn test_generate_missing_ap_users_migration() -> anyhow::Result<()> {
    // Create a test database pool without running migrations
    let pool = super::util::db::new_test_pool().await;
    let base_url = Url::parse("http://localhost:3000")?;

    // First, run up to and including the migration that creates the ap_users table
    // and adds the ap_user_id column, but don't run the Rust
    // migration that generates AP users yet
    #[allow(clippy::inconsistent_digit_grouping)]
    let sql_migration_version = Some(2025_01_27_102308);
    db::migrate(&pool, &base_url, sql_migration_version).await?;

    // Create a user without an associated ap_user_id
    // Don't use static queries here since they don't reflect the newest schema
    let mut conn = pool.acquire().await?;
    let user_id: Uuid = sqlx::query(
        r"
        insert into users (username)
        values ($1)
        returning id
        ",
    )
    .bind("testuser")
    .fetch_one(&mut *conn)
    .await?
    .get(0);

    // Verify the user was created without an ap_user_id
    let ap_user_id: Option<Uuid> = sqlx::query("select ap_user_id from users where id = $1")
        .bind(user_id)
        .fetch_one(&mut *conn)
        .await?
        .get(0);

    assert!(
        ap_user_id.is_none(),
        "User should not have an ap_user_id before migration"
    );

    // Now run the Rust migration that generates AP users for existing users
    db::migrate(&pool, &base_url, None).await?;

    // Verify that the user now has an associated ap_user_id
    let ap_user_id: Uuid = sqlx::query!("select ap_user_id from users where id = $1", user_id)
        .fetch_one(&mut *conn)
        .await?
        .ap_user_id;

    // Verify that the AP user was created with correct data
    let row = sqlx::query!(
        "select ap_id, username, inbox_url from ap_users where id = $1",
        ap_user_id
    )
    .fetch_one(&mut *conn)
    .await?;

    assert_eq!(
        row.username, "testuser",
        "AP user should have the same username"
    );
    assert_eq!(
        row.ap_id,
        base_url.join("/ap/user/")?.join(&row.username)?.to_string(),
        "AP ID should be correctly formatted"
    );
    assert_eq!(
        row.inbox_url,
        base_url.join("/ap/inbox")?.to_string(),
        "Inbox URL should be correctly formatted"
    );

    Ok(())
}
