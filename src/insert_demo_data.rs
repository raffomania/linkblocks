use anyhow::{anyhow, Result};
use fake::{Fake, Faker};
use garde::Validate;
use rand::seq::{IteratorRandom, SliceRandom};
use sqlx::PgPool;
use strum::IntoEnumIterator;

use crate::{
    db::{self, bookmarks},
    schemas::{
        bookmarks::CreateBookmark,
        links::{CreateLink, ReferenceType},
        lists::CreateList,
        notes::CreateNote,
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
    let mut bookmarks = Vec::new();
    let mut notes = Vec::new();

    for user in users.iter() {
        for _ in 0..10 {
            let title: String = fake::faker::lorem::en::Words(1..10)
                .fake::<Vec<_>>()
                .join(" ");
            let create_list = CreateList { title };
            lists.push(db::lists::insert(&mut tx, user.id, create_list).await?);
        }

        for _ in 0..100 {
            let tld: String = fake::faker::internet::en::DomainSuffix().fake();
            let word: String = fake::faker::lorem::en::Word().fake();
            let title: String = fake::faker::lorem::en::Sentence(1..5).fake();
            let create_bookmark = CreateBookmark {
                url: format!("https://{word}.{tld}"),
                title,
            };
            create_bookmark.validate(&())?;

            let bookmark = bookmarks::insert(&mut tx, user.id, create_bookmark).await?;
            bookmarks.push(bookmark);
        }

        for _ in 0..30 {
            let content: Vec<_> = fake::faker::lorem::en::Paragraphs(1..3).fake();
            let create_note = CreateNote {
                content: content.join("\n\n"),
            };
            let note = db::notes::insert(&mut tx, user.id, create_note).await?;

            notes.push(note);
        }
    }

    for user in users.iter() {
        for _ in 0..200 {
            let src_ref_type = random_reference_type()?;
            let dest_ref_type = random_reference_type()?;
            let src_id = match src_ref_type {
                ReferenceType::Bookmark => {
                    bookmarks
                        .choose(&mut rand::thread_rng())
                        .ok_or(anyhow!("Found no random bookmark to put into a link"))?
                        .id
                }
                ReferenceType::Note => {
                    notes
                        .choose(&mut rand::thread_rng())
                        .ok_or(anyhow!("Found no random note to put into a link"))?
                        .id
                }
                ReferenceType::List => {
                    lists
                        .choose(&mut rand::thread_rng())
                        .ok_or(anyhow!("Found no random list to put into a link"))?
                        .id
                }
            };
            let dest_id = match dest_ref_type {
                ReferenceType::Bookmark => {
                    bookmarks
                        .choose(&mut rand::thread_rng())
                        .ok_or(anyhow!("Found no random bookmark to put into a link"))?
                        .id
                }
                ReferenceType::Note => {
                    notes
                        .choose(&mut rand::thread_rng())
                        .ok_or(anyhow!("Found no random note to put into a link"))?
                        .id
                }
                ReferenceType::List => {
                    lists
                        .choose(&mut rand::thread_rng())
                        .ok_or(anyhow!("Found no random list to put into a link"))?
                        .id
                }
            };

            let create_link = CreateLink {
                src_id,
                src_ref_type,
                dest_id,
                dest_ref_type,
            };
            db::links::insert(&mut tx, user.id, create_link).await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

fn random_reference_type() -> Result<ReferenceType> {
    ReferenceType::iter()
        .choose(&mut rand::thread_rng())
        .ok_or(anyhow!("Failed to choose a random reference type"))
}
