use cargo_metadata::MetadataCommand;
use serde_json::Value;
use std::{env, path::PathBuf};

const DEFAULT_ROUTES_FOLDER: &str = "src/routes";

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let pwd = env::var("PWD").unwrap();

    let metadata = MetadataCommand::new()
        .current_dir(&pwd)
        .no_deps()
        .exec()
        .unwrap();

    let Some(root_package) = metadata.root_package() else {
        panic!("No root package found in cargo.toml");
    };

    let config = root_package
        .metadata
        .get("axum-fs-router")
        .unwrap_or(&Value::Null);

    let routes_dir = match config.get("routes_dir") {
        Some(routes_dir) => match routes_dir.clone() {
            Value::String(routes_dir) => routes_dir,
            _ => panic!("Invalid type for routes_dir in [package.metadata.axum_fs_router]"),
        },
        None => String::from(DEFAULT_ROUTES_FOLDER),
    };

    let absolute_routes_dir = PathBuf::from(metadata.workspace_root).join(routes_dir);

    println!(
        "cargo:rustc-env=ROUTES_DIR={}",
        absolute_routes_dir.display()
    );
    println!("cargo:rerun-if-changed={}", absolute_routes_dir.display());
}
