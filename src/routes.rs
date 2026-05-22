use eyre::{Result, eyre};
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};
use syn::Ident;

#[derive(Debug)]
pub struct Routes {
    inner: Vec<Segment>,
}

#[derive(Debug)]
pub enum Segment {
    Folder {
        name: String,
        route_path: String,
        module_path: String,
        sub: Routes,
    },
    Handler {
        name: String,
        module_path: String,
    },
}

impl Routes {
    pub fn new(folder: impl Into<PathBuf>) -> Result<Self> {
        let folder = folder.into();
        let mut inner = Vec::new();

        let mut entries =
            fs::read_dir(&folder)?.collect::<Result<Vec<DirEntry>, std::io::Error>>()?;
        entries.sort_by_key(DirEntry::file_name);

        for entry in entries {
            let file_type = entry.file_type()?;
            let entry_path = entry.path();

            let file_path = entry.file_name().into_string().map_err(|_| {
                eyre!(
                    "Invalid filename {file}",
                    file = entry.file_name().display()
                )
            })?;

            if file_type.is_dir() {
                let name = String::from(file_path.trim_matches(|char| char == '{' || char == '}'));

                let sub = Self::new(&entry_path)?;

                inner.push(Segment::Folder {
                    name,
                    route_path: file_path,
                    module_path: entry_path.display().to_string(),
                    sub,
                });
            } else {
                let name = file_path.replace(".rs", "");

                match name.as_str() {
                    "any" | "connect" | "delete" | "get" | "head" | "options" | "patch"
                    | "post" | "put" | "trace" => (),

                    _ => continue,
                }

                inner.push(Segment::Handler {
                    name,
                    module_path: entry_path.display().to_string(),
                });
            }
        }

        Ok(Self { inner })
    }
}

impl ToTokens for Segment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Segment::Folder {
                name,
                sub,
                module_path,
                ..
            } => {
                let name = Ident::new(name, Span::call_site());

                tokens.extend(quote! {
                    #[path = #module_path]
                    pub mod #name { #sub }
                });
            }
            Segment::Handler { name, module_path } => {
                let name = Ident::new(name, Span::call_site());

                tokens.extend(quote! {
                    #[path = #module_path]
                    pub mod #name;
                });
            }
        }
    }
}

impl ToTokens for Routes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let segments = &self.inner;
        tokens.extend(quote! { #(#segments)* });
    }
}

impl Segment {
    fn collect_handler_files(&self, tracked_files: &mut Vec<String>) {
        match self {
            Segment::Folder { sub, .. } => sub.collect_handler_files(tracked_files),
            Segment::Handler { module_path, .. } => tracked_files.push(module_path.clone()),
        }
    }

    fn to_handler(&self) -> TokenStream {
        match self {
            Segment::Folder {
                name,
                route_path,
                sub,
                ..
            } => {
                let relative_route_path = format!("/{route_path}");
                let sub_router = sub.to_router();
                let module = Ident::new(name, Span::call_site());

                quote! { .nest(#relative_route_path, {
                    use #module::*;
                    #sub_router
                }) }
            }

            Segment::Handler { name, .. } => {
                let method = Ident::new(name, Span::call_site());

                quote! { .route("/", ::axum::routing::#method(#method::handler)) }
            }
        }
    }
}

impl Routes {
    fn collect_handler_files(&self, tracked_files: &mut Vec<String>) {
        for segment in &self.inner {
            segment.collect_handler_files(tracked_files);
        }
    }

    pub fn tracked_handler_files(&self) -> Vec<String> {
        let mut tracked_files = Vec::new();
        self.collect_handler_files(&mut tracked_files);
        tracked_files
    }

    pub fn to_router(&self) -> TokenStream {
        let streams = self
            .inner
            .iter()
            .map(Segment::to_handler)
            .collect::<Vec<TokenStream>>();

        quote! { ::axum::routing::Router::new()#(#streams)* }
    }
}
