use anyhow::Context;
use link_preview::{html_from_bytes, LinkPreview};
use reqwest;

use crate::db::metadata::InsertMetadata;

pub async fn get_metadata(url: &str) -> InsertMetadata {
    let html = reqwest::get(url)
        .await
        .context("Failed to scrawl given url")
        .unwrap()
        .bytes()
        .await
        .context("Faield to convert to bytes")
        .unwrap();
    let preview = LinkPreview::from(
        html_from_bytes(&html)
            .context("Failed to load preview")
            .unwrap(),
    );
    let metadata_title = match preview.title {
        Some(title) => title,
        None => url.split("/").last().unwrap().to_string(),
    };
    let metadata_image_url = match preview.image_url {
        Some(image_url) => Some(image_url.to_string()),
        None => None,
    };
    let metadata = InsertMetadata {
        metadata_title,
        metadata_description: preview.description,
        metadata_image_url,
    };
    metadata
}
