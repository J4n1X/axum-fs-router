# Axum Filesystem Router

This crate can be used to have a filesystem router.

> **NOTE:** This is an early version of the project.

> You need to have axum installed.

Look at the doc comments of the macros to know what they do.

## Configuration

You can configure which directory is traversed by adding metadata in your `Cargo.toml`:

```toml
[package.metadata.axum-fs-router]
routes-dir = "src/routes"
```

`routes-dir` can be any path relative to your crate root.

Equivalent snake_case keys are also accepted:

```toml
[package.metadata.axum_fs_router]
routes_dir = "src/routes"
```

Rebuild behavior:

- Changes to tracked handler files are picked up automatically.
- Changes to `Cargo.toml` are picked up automatically.
- Newly added or removed handler files may require any normal Rust file change to trigger recompilation.

## Example

Due to some issues with rust-analyzer you should put the `traverse_routes!()` call into a seperate file. This file currently needs to be at the root level of the `src` directory.

```rs
// `src/routes.rs`
axum_fs_router::traverse_routes!();
```

You can then use it as follows. `router!()` returns a normal `axum::Router`.

```rs
use crate::routes::__generated_routes;

let app = axum_fs_router::router!()
    .layer(CookieManagerLayer::new())
    .with_state(10_u8);
```

Handlers can be at paths like these:

```rs
// `src/routes/foo/bar/get.rs`
pub async fn handler() -> &'static str {
    "Hello world from `/foo/bar` !"
}
```

```rs
// `src/routes/{some_id}/post.rs`
use axum::extract::Path;
use uuid::Uuid;

pub async fn handler(Path(id): Path<Uuid>) {
    log::info!("{id}")
}
```
