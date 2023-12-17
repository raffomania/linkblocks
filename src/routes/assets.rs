use std::path::PathBuf;

use crate::app_error::Result;
use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::Path,
    http::{header, HeaderMap},
};
use include_dir::{include_dir, Dir};

static ASSETS_DIR: Dir = include_dir!("assets");

#[debug_handler]
pub async fn assets(Path(path): Path<PathBuf>) -> Result<(HeaderMap, &'static [u8])> {
    // TODO write tests ensuring that all assets have guessable mime types
    let ext = path
        .extension()
        .ok_or(anyhow!("Included assets need an extension"))?
        .to_str()
        .ok_or(anyhow!("Path extension had invalid unicode"))?;

    let mime_type = mime_guess::from_ext(ext).first_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

    let headers =
        HeaderMap::from_iter([(header::CONTENT_TYPE, mime_type.to_string().parse()?)].into_iter());

    let body = ASSETS_DIR
        .get_file(path)
        .map(|f| f.contents())
        .ok_or(anyhow!("not found"))?;

    Ok((headers, body))
}
