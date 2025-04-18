use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{link::LinkType, object::NoteType},
    protocol::verification::{verify_domains_match, verify_is_remote_object},
    traits::Object,
};
use anyhow::anyhow;
use garde::Validate;
use serde::{Deserialize, Serialize};
use url::Url;

use super::user::UserJson;
use crate::{
    db::{self, bookmarks::InsertBookmark},
    forms::bookmarks::CreateBookmark,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkJson {
    // pub id: ObjectId<db::Bookmark>,
    #[serde(rename = "type")]
    pub kind: NoteType,
    pub attributed_to: ObjectId<db::ApUser>,
    pub name: Option<String>,
    #[serde(default)]
    pub(crate) attachments: Vec<Link>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Link {
    href: Url,
    media_type: Option<String>,
    #[serde(rename = "type")]
    kind: LinkType,
}

impl TryFrom<BookmarkJson> for InsertBookmark {
    type Error = anyhow::Error;

    fn try_from(value: BookmarkJson) -> Result<Self, Self::Error> {
        let first_attachment = value.attachments.first();
        let url = if let Some(attachment) = first_attachment.cloned() {
            Some(attachment.href)
        } else {
            None
        }
        .ok_or_else(|| anyhow!("Missing URL"))?;
        let create_bookmark = InsertBookmark {
            url: url.to_string(),
            title: value.name.ok_or_else(|| anyhow!("Missing title"))?,
        };

        // TODO how to validate InsertBookmark?
        // ideally, we'd find a single point for the validation rules that
        // now live in CreateBookmark

        Ok(create_bookmark)
    }
}

#[async_trait::async_trait]
impl Object for db::Bookmark {
    type DataType = super::Context;
    type Kind = BookmarkJson;
    type Error = anyhow::Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        todo!()
    }

    async fn into_json(self, context: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let mut tx = context.db_pool.begin().await?;
        let author = db::ap_users::read_by_local_user_id(&mut tx, self.user_id).await?;
        let attachments = vec![Link {
            href: self.url.parse()?,
            media_type: todo!(),
            kind: LinkType::Link,
        }];
        Ok(BookmarkJson {
            id: todo!(),
            kind: NoteType::Note,
            attributed_to: author.ap_id,
            name: Some(self.title),
            attachments,
        })
    }
    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        verify_is_remote_object(&json.id, data)?;
        // TODO see validation todo above
        // InsertBookmark::try_from(json.clone())?.validate()?;
        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let insert_bookmark = json.try_into()?;

        let mut tx = data.db_pool.begin().await?;
        let new_user = db::bookmarks::upsert(&mut tx, insert_bookmark).await?;
        tx.commit().await?;
        Ok(new_user)
    }
}
