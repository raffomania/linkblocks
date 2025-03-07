use railwind::{Source, SourceOptions};
use std::{env, path::Path};

fn main() {
  // Without this, adding only a migration will not trigger a re-build
  // https://docs.rs/sqlx/latest/sqlx/macro.migrate.html#stable-rust-cargo-build-script
  println!("cargo:rerun-if-changed=migrations");

  println!("cargo:rerun-if-changed=templates");

  let out_dir = env::var("OUT_DIR").unwrap();

  let dest_path = Path::new(&out_dir).join("railwind.css");

  let paths: Vec<_> = walkdir::WalkDir::new("templates")
    .into_iter()
    .map(|e| e.expect("Error while searching for templates"))
    .filter(|e| e.file_type().is_file())
    .map(|entry| entry.into_path())
    .collect();

  let mut railwind_sources: Vec<_> = paths
    .iter()
    .map(|p| SourceOptions {
      input: p,
      option: railwind::CollectionOptions::Html,
    })
    .collect();

  let paths: Vec<_> = walkdir::WalkDir::new("src/views")
    .into_iter()
    .map(|e| e.expect("Error while searching for templates"))
    .filter(|e| e.file_type().is_file())
    .map(|entry| entry.into_path())
    .collect();

  railwind_sources.extend(paths.iter().map(|p| SourceOptions {
    input: p,
    option: railwind::CollectionOptions::String,
  }));

  let source = Source::Files(railwind_sources);
  railwind::parse_to_file(source, dest_path.to_str().unwrap(), false, &mut Vec::new());
}
