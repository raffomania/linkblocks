use anyhow::Result;
use garde::Validate;
use sqlx::PgPool;

use crate::{
    db::{self, bookmarks},
    schemas::{bookmarks::CreateBookmark, users::CreateUser},
};

pub async fn insert_demo_data(pool: &PgPool) -> Result<()> {
    let mut tx = pool.begin().await?;

    let create_user = CreateUser {
        username: "demouser".to_string(),
        password: "demopassword".to_string(),
    };
    create_user.validate(&())?;

    let user = db::users::insert(&mut tx, create_user).await?;
    for _ in 0..100 {
        let create_bookmark = CreateBookmark {
            url: "https://www.rafa.ee".to_string(),
            user_id: user.id,
        };
        create_bookmark.validate(&())?;

        bookmarks::insert(&mut tx, create_bookmark).await?;
    }

    tx.commit().await?;

    Ok(())
}
