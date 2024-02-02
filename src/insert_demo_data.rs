use anyhow::Result;
use fake::{Fake, Faker};
use garde::Validate;
use sqlx::PgPool;

use crate::{
    db::{self, bookmarks},
    schemas::{
        bookmarks::CreateBookmark,
        links::{CreateLink, ReferenceType},
        lists::CreateList,
        users::CreateUser,
    },
};

pub async fn insert_demo_data(
    pool: &PgPool,
    dev_user_credentials: Option<CreateUser>,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    let mut users = Vec::new();
    for _ in 0..5 {
        let random_id: u64 = Faker.fake();
        let create_user = CreateUser {
            username: format!("demouser{random_id}"),
            password: "testpassword".to_string(),
        };

        users.push(db::users::insert(&mut tx, create_user).await?)
    }

    if let Some(create_dev_user) = dev_user_credentials {
        users.push(db::users::create_user_if_not_exists(&mut tx, create_dev_user).await?);
    }

    let mut lists = Vec::new();
    for user in users.iter() {
        for _ in 0..10 {
            let title: String = fake::faker::lorem::en::Sentence(1..10).fake();
            let create_list = CreateList { title };
            lists.push(db::lists::insert(&mut tx, user.id, create_list).await?);
        }
    }

    for user in users.iter() {
        for list in lists.iter() {
            for _ in 0..20 {
                let tld: String = fake::faker::internet::en::DomainSuffix().fake();
                let word: String = fake::faker::lorem::en::Word().fake();
                let create_bookmark = CreateBookmark {
                    url: format!("https://{word}.{tld}"),
                };
                create_bookmark.validate(&())?;

                let bookmark = bookmarks::insert(&mut tx, user.id, create_bookmark).await?;

                let create_link = CreateLink {
                    src_id: list.id,
                    src_ref_type: ReferenceType::List,
                    dest_id: bookmark.id,
                    dest_ref_type: ReferenceType::Bookmark,
                };

                db::links::insert(&mut tx, user.id, create_link).await?;
            }
        }
    }

    tx.commit().await?;

    Ok(())
}
