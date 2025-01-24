use anyhow::{anyhow, Result};
use fake::Fake;
use rand::{seq::SliceRandom, Rng};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::{self, bookmarks::InsertBookmark},
    forms::{
        links::CreateLink,
        lists::CreateList,
        users::{CreateOidcUser, CreateUser},
    },
};

pub async fn insert_demo_data(
    pool: &PgPool,
    dev_user_credentials: Option<CreateUser>,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    let mut users = Vec::new();
    for _ in 0..5 {
        let email: Option<String> = fake::faker::internet::en::SafeEmail().fake();
        let username: String = fake::faker::name::en::Name().fake();
        let username = username.to_lowercase();
        let username = username.replace(' ', "");
        if let Some(email) = email {
            let create_oidc_user = CreateOidcUser {
                oidc_id: Uuid::new_v4().to_string(),
                email,
                username,
            };

            users.push(db::users::insert_oidc(&mut tx, create_oidc_user).await?);
        } else {
            let create_user = CreateUser {
                username,
                password: "testpassword".to_string(),
            };

            users.push(db::users::insert(&mut tx, create_user).await?);
        }
    }

    if let Some(create_dev_user) = dev_user_credentials {
        users.push(db::users::create_user_if_not_exists(&mut tx, create_dev_user).await?);
    }

    let mut bookmarks = Vec::new();
    let mut lists = Vec::new();

    for user in &users {
        for _ in 0..500 {
            let tld: String = fake::faker::internet::en::DomainSuffix().fake();
            let word: String = fake::faker::lorem::en::Word().fake();
            let title: String = fake::faker::lorem::en::Words(1..5)
                .fake::<Vec<_>>()
                .join(" ");
            let insert_bookmark = InsertBookmark {
                url: format!("https://{word}.{tld}"),
                title,
            };

            let bookmark = db::bookmarks::insert(&mut tx, user.id, insert_bookmark).await?;
            bookmarks.push(bookmark);
        }

        for _ in 0..100 {
            let content: Option<Vec<_>> = fake::faker::lorem::en::Paragraphs(1..3).fake();
            let city: String = fake::faker::address::en::CityName().fake();
            let noun: String = fake::faker::company::en::BsNoun().fake();
            let title = format!("{city} {noun}");
            let create_list = CreateList {
                title,
                content: content.map(|c| c.join("\n\n")),
                private: fake::Faker.fake(),
            };
            let list = db::lists::insert(&mut tx, user.id, create_list).await?;

            if fake::faker::boolean::en::Boolean(10).fake() {
                db::lists::set_pinned(&mut tx, list.id, false).await?;
            }

            lists.push(list);
        }
    }

    for user in users {
        for _ in 0..1000 {
            let src = lists
                .choose(&mut rand::thread_rng())
                .ok_or(anyhow!("Found no random list to put into a link"))?
                .id;
            let dest = random_link_reference(&bookmarks, &lists)?;

            let create_link = CreateLink { src, dest };
            db::links::insert(&mut tx, user.id, create_link).await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

fn random_link_reference(bookmarks: &[db::Bookmark], lists: &[db::List]) -> Result<Uuid> {
    Ok(match rand::thread_rng().gen_range(0..=1) {
        0 => {
            bookmarks
                .choose(&mut rand::thread_rng())
                .ok_or(anyhow!("Found no random bookmark to put into a link"))?
                .id
        }
        1 => {
            lists
                .choose(&mut rand::thread_rng())
                .ok_or(anyhow!("Found no random list to put into a link"))?
                .id
        }
        _ => unreachable!(),
    })
}
