use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::Response,
};

/// Embedded dashboard frontend assets.
///
/// Built from `dashboard-ui/` via `npm run build`.
/// The output directory `dashboard-ui/dist/` is embedded at compile time.
#[derive(rust_embed::RustEmbed)]
#[folder = "dashboard-ui/dist"]
#[include = "*.html"]
#[include = "*.js"]
#[include = "*.css"]
struct Assets;

fn serve_embedded(path: &str) -> Response {
    match Assets::get(path) {
        Some(content) => {
            let body = Body::from(content.data.to_vec());
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
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
    let asset_path = if clean_path.is_empty() {
        return serve_root().await;
    } else {
        clean_path
    };

    match Assets::get(asset_path) {
        Some(content) => {
            let body = Body::from(content.data.to_vec());
            let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap()
        }
        None => {
            // SPA fallback: serve index.html for unknown paths
            serve_embedded("index.html")
        }
    }
}
