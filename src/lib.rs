use crate::routes::Routes;
use quote::quote;
use std::{env, path::PathBuf};

mod routes;

/// Returns the configred path through the rustc env variables provided by the build.rs file
fn configured_routes_folder() -> PathBuf {
    PathBuf::from(env!("ROUTES_DIR"))
}

/// An internal wrapper function to return a [`Result`]
fn traverse_routes_wrapper() -> eyre::Result<proc_macro2::TokenStream> {
    let routes_folder = configured_routes_folder();
    let routes = Routes::new(routes_folder)?;
    let tracked_files = routes.tracked_handler_files();

    Ok(quote! {
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
    let routes_folder = configured_routes_folder();

    let routes = Routes::new(routes_folder)?;
    let router_tokens = routes.to_router();

    let tracked_files = routes.tracked_handler_files();

    Ok(quote! {
       {
            #(const _: &str = include_str!(#tracked_files);)*
            use __generated_routes::*;

            #router_tokens
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
