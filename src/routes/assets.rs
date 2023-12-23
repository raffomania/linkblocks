use std::path::PathBuf;

use crate::app_error::{AppError, Result};
use anyhow::{anyhow, Context};
use axum::{
    debug_handler,
    extract::Path,
    http::{header, HeaderMap},
};
use include_dir::{include_dir, Dir};
use mime_guess::Mime;

static ASSETS_DIR: Dir = include_dir!("assets");

#[debug_handler]
pub async fn assets(Path(path): Path<PathBuf>) -> Result<(HeaderMap, &'static [u8])> {
    let body = ASSETS_DIR
        .get_file(&path)
        .map(|f| f.contents())
        .ok_or(AppError::NotFound())?;

    let mime_type = get_mime(&path)?;

    let headers =
        HeaderMap::from_iter([(header::CONTENT_TYPE, mime_type.to_string().parse()?)].into_iter());

    Ok((headers, body))
}

fn get_mime(path: &PathBuf) -> Result<Mime> {
    let ext = path
        .extension()
        .ok_or(anyhow!("Included assets need an extension"))?
        .to_str()
        .ok_or(anyhow!("Path extension had invalid unicode"))?;

    Ok(mime_guess::from_ext(ext)
        .first()
        .context("No mime type guessed")?)
}

#[cfg(test)]
mod tests {
    use include_dir::Dir;

    use super::Result;
    use super::{get_mime, ASSETS_DIR};

    #[test]
    fn all_assets_have_a_mime_type() -> Result<()> {
        fn check_dir(dir: &Dir) -> Result<()> {
            for asset in dir.files() {
                get_mime(&asset.path().to_path_buf())?;
            }

            for dir in ASSETS_DIR.dirs() {
                check_dir(dir)?;
            }

            Ok(())
        }

        check_dir(&ASSETS_DIR)?;

        Ok(())
    }
}
