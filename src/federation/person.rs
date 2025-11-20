//! Adapter to make [`db::ApUser`] compatible with the
//! [`activitypub_federation`] crate

use activitypub_federation::{
    fetch::object_id::ObjectId,
    kinds::actor::PersonType,
    protocol::{
        public_key::PublicKey,
        verification::{verify_domains_match, verify_is_remote_object},
    },
    traits::{Actor, Object},
};
use anyhow::{Context, Result};
use garde::Validate;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    date_time::time_to_chrono,
    db,
    forms::ap_users::CreateApUser,
    response_error::{ResponseError, into_option},
};

/// Users as we receive from and send to other instances.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: ObjectId<db::ApUser>,
    #[serde(rename = "type")]
    pub kind: PersonType,
    pub preferred_username: String,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub inbox: Url,
    pub outbox: Url,
    pub public_key: PublicKey,
    /// "Identifies one or more links to representations of the object"
    pub url: Url,
}

#[async_trait::async_trait]
impl Object for db::ApUser {
    type DataType = super::Context;
    type Kind = Person;
    type Error = ResponseError;

    fn last_refreshed_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        Some(time_to_chrono(self.last_refreshed_at))
    }

    async fn read_from_id(object_id: Url, data: &super::Data) -> Result<Option<Self>, Self::Error> {
        let mut tx = data.db_pool.begin().await?;
        let user = db::ap_users::read_by_ap_id(&mut tx, &object_id).await;
        into_option(user)
    }

    async fn into_json(self, context: &super::Data) -> Result<Self::Kind, Self::Error> {
        let public_key = self.public_key();
        let url = context.base_url.join("/user/")?.join(&self.username)?;
        let outbox = context
            .base_url
            .join("/ap/outbox/")?
            .join(&self.id.to_string())?;
        Ok(Person {
            id: self.ap_id,
            name: self.display_name,
            preferred_username: self.username,
            kind: PersonType::Person,
            inbox: self.inbox_url,
            outbox,
            public_key,
            summary: self.bio,
            url,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        data: &super::Data,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        verify_is_remote_object(&json.id, data)?;
        CreateApUser::new_remote(json.clone())?
            .validate()
            .context("Invalid input")?;
        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &super::Data) -> Result<Self, Self::Error> {
        let create_user = CreateApUser::new_remote(json)?;
        let mut tx = data.db_pool.begin().await?;
        let new_user = db::ap_users::upsert(&mut tx, create_user).await?;
        tx.commit().await?;
        Ok(new_user)
    }
}

impl Actor for db::ApUser {
    fn id(&self) -> Url {
        self.ap_id.inner().clone()
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        self.private_key.as_ref().map(|s| s.expose_secret().clone())
    }

    fn inbox(&self) -> Url {
        self.inbox_url.clone()
    }
}
