use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::AcceptType,
    traits::{ActivityHandler, Actor},
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    db,
    federation::{activity, follow::Follow},
    response_error::{ResponseError, ResponseResult},
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
    actor: ObjectId<db::ApUser>,
    object: Follow,
    #[serde(rename = "type")]
    kind: AcceptType,
    id: Url,
}

impl Accept {
    pub async fn send(
        actor: &db::ApUser,
        object: Follow,
        context: &Data<super::context::Context>,
    ) -> ResponseResult<()> {
        let id = super::activity::generate_id(context)?;
        let follower = object.actor.dereference(context).await?;
        let accept = Accept {
            actor: actor.ap_id.clone(),
            object,
            kind: AcceptType::Accept,
            id,
        };
        activity::send(
            actor,
            accept,
            vec![follower.shared_inbox_or_inbox()],
            context,
        )
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Accept {
    type DataType = super::context::Context;
    type Error = ResponseError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Err(ResponseError::NotFound)
    }
}
