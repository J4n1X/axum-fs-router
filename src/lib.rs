use crate::routes::Routes;
use quote::quote;
use std::{env, fs, path::PathBuf};
use toml::Value;

mod routes;

const DEFAULT_ROUTES_FOLDER: &str = "src/routes";

fn configured_routes_folder() -> eyre::Result<(PathBuf, PathBuf)> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let manifest_path = PathBuf::from(&manifest_dir).join("Cargo.toml");

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: Value = toml::from_str(&manifest_content)?;

    let configured_path = manifest
        .get("package")
        .and_then(|pkg| pkg.get("metadata"))
        .and_then(|meta| meta.get("axum-fs-router").or_else(|| meta.get("axum_fs_router")))
        .and_then(|cfg| cfg.get("routes-dir").or_else(|| cfg.get("routes_dir")))
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_ROUTES_FOLDER);

    let absolute_routes_path = PathBuf::from(manifest_dir).join(configured_path);

    Ok((absolute_routes_path, manifest_path))
}

/// An internal wrapper function to return a [`Result`]
fn traverse_routes_wrapper() -> eyre::Result<proc_macro2::TokenStream> {
    let (routes_folder, manifest_path) = configured_routes_folder()?;
    let routes = Routes::new(routes_folder)?;
    let tracked_files = routes.tracked_handler_files();
    let manifest_path = manifest_path.display().to_string();

    Ok(quote! {
        const _: &str = include_str!(#manifest_path);
        #(const _: &str = include_str!(#tracked_files);)*
        pub mod __generated_routes { #routes }
    })
}

/// This macro will traverse the configured routes folder in your project (defaults to `src/routes`) and search for files any of these names:
///
/// - `any.rs`
/// - `connect.rs`
/// - `delete.rs`
/// - `get.rs`
/// - `head.rs`
/// - `options.rs`
/// - `patch.rs`
/// - `post.rs`
/// - `put.rs`
/// - `trace.rs`
///
/// If a file is found recursively, a module will be created for it.
///
/// > **NOTE:** You should put the [`traverse_routes`] invocation into the root of the project and into a seperate file. Due to some glitchiness with rust-analyzer it will alter the code if a file under the routes folder is renamed.
#[proc_macro]
pub fn traverse_routes(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match traverse_routes_wrapper() {
        Ok(tokens) => tokens,
        Err(error) => {
            let error = format!("{error:?}");
            quote! { compiler_error!(#error) }
        }
    }
    .into()
}

/// An internal wrapper function to return a [`Result`]
fn router_wrapper() -> eyre::Result<proc_macro2::TokenStream> {
    let (routes_folder, manifest_path) = configured_routes_folder()?;
    let routes = Routes::new(routes_folder)?;
    let router = routes.to_router();
    let tracked_files = routes.tracked_handler_files();
    let manifest_path = manifest_path.display().to_string();

    Ok(quote! {
       {
            const _: &str = include_str!(#manifest_path);
            #(const _: &str = include_str!(#tracked_files);)*
            use __generated_routes::*;

            #router
       }
    })
}

/// This macro returns an axum `Router` that you can use however you want. You can merge it with an existing router or just use it as a base.
///
/// > **NOTE:** This macro expects a `pub async fn handler()` to be exported from each of the handler files. It needs to be async as thats what axum wants. You can use any extractors you would normally use in an axum handler.
#[proc_macro]
pub fn router(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match router_wrapper() {
        Ok(tokens) => tokens,
        Err(error) => {
            let error = format!("{error:?}");
            quote! { compiler_error!(#error) }
        }
    }
    .into()
}
