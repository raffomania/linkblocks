use std::{env, path::Path};

use railwind::{Source, SourceOptions};
use regex::Regex;

fn main() {
    // Without this, adding only a migration will not trigger a re-build
    // https://docs.rs/sqlx/latest/sqlx/macro.migrate.html#stable-rust-cargo-build-script
    println!("cargo:rerun-if-changed=migrations");

    println!("cargo:rerun-if-changed=src/views");

    let out_dir = env::var("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("railwind.css");

    let paths: Vec<_> = walkdir::WalkDir::new("src/views")
        .into_iter()
        .map(|e| e.expect("Error while searching for views"))
        .filter(|e| e.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect();

    let sources = paths
        .iter()
        .map(|p| SourceOptions {
            input: p,
            option: railwind::CollectionOptions::Regex(
                Regex::new(r#"class[\n\s\(]*"([^"]+)""#).unwrap(),
            ),
        })
        .collect();

    let source = Source::Files(sources);
    railwind::parse_to_file(source, dest_path.to_str().unwrap(), false, &mut Vec::new());
}
