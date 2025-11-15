use std::fmt::Debug;

use activitypub_federation::{
    activity_queue::queue_activity,
    protocol::context::WithContext,
    traits::{ActivityHandler, Actor},
};
use serde::Serialize;
use url::Url;
use uuid::Uuid;

use crate::federation::context::Data;

pub async fn send<Activity, ActorType: Actor>(
    actor: &ActorType,
    activity: Activity,
    recipients: Vec<Url>,
    context: &Data,
) -> Result<(), <Activity as ActivityHandler>::Error>
where
    Activity: ActivityHandler + Serialize + Debug + Send + Sync,
    <Activity as ActivityHandler>::Error: From<activitypub_federation::error::Error>,
{
    let activity = WithContext::new_default(activity);
    queue_activity(&activity, actor, recipients, context).await?;
    Ok(())
}

pub fn generate_id(context: &Data) -> Result<Url, url::ParseError> {
    context
        .base_url
        .join("/ap/activity/")?
        .join(&Uuid::new_v4().to_string())
}
