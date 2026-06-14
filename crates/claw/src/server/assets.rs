use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use rust_embed::RustEmbed;

/// Embedded dashboard frontend assets.
///
/// Built from `dashboard-ui/` via `npm run build`.
/// The output directory `dashboard-ui/dist/` is embedded at compile time.
#[derive(RustEmbed)]
#[folder = "dashboard-ui/dist"]
struct Assets;

fn serve_embedded(path: &str) -> Response {
    match Assets::get(path) {
        Some(content) => {
            let body = Body::from(content.data.to_vec());
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let mut builder = Response::builder().header(header::CONTENT_TYPE, mime.as_ref());
            // index.html must always be revalidated so clients pick up new deploys;
            // hashed assets under assets/ can be cached aggressively.
            if path == "index.html" {
                builder = builder.header(header::CACHE_CONTROL, "no-cache");
            } else {
                builder = builder.header(header::CACHE_CONTROL, "public, max-age=86400");
            }
            builder
                .body(body)
                .expect("Assets response builder never fails with valid body")
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .expect("Assets 404 response builder never fails"),
    }
}

/// Serve the SPA root (`index.html`).
pub async fn serve_root() -> Response {
    serve_embedded("index.html")
}

/// Serve static assets from the embedded frontend.
///
/// For SPA routing, any path that doesn't match an embedded file
/// falls back to `index.html`.
pub async fn serve_assets(Path(path): Path<String>) -> Response {
    let clean_path = path.trim_start_matches('/');
    if clean_path.is_empty() {
        return serve_root().await;
    }
    // For known embedded files, serve directly; otherwise SPA fallback.
    if Assets::get(clean_path).is_some() {
        serve_embedded(clean_path)
    } else {
        serve_embedded("index.html")
    }
}
