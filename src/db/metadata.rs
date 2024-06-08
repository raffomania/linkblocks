use super::AppTx;
use crate::response_error::ResponseResult;
use serde::Deserialize;
use sqlx::{query_as, FromRow};
use uuid::Uuid;

#[derive(FromRow, Debug, Deserialize)]
pub struct Metadata {
    pub id: Uuid,
    pub metadata_title: String,
    pub metadata_description: Option<String>,
    pub metadata_image_url: Option<String>,
}

#[derive(FromRow, Debug)]
pub struct InsertMetadata {
    pub metadata_title: String,
    pub metadata_description: Option<String>,
    pub metadata_image_url: Option<String>,
}

pub async fn insert(tx: &mut AppTx, metadata: InsertMetadata) -> ResponseResult<Metadata> {
    let metadata = query_as!(
        Metadata,
        r#"
        insert into metadata
        (metadata_title,metadata_description, metadata_image_url)
        values ($1, $2, $3)
        returning *
        "#,
        metadata.metadata_title,
        metadata.metadata_description,
        metadata.metadata_image_url
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(metadata)
}
