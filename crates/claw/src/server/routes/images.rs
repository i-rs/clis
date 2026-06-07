use axum::extract::Path;

/// Serve generated images from ~/.i-rs/claw/images/.
pub async fn serve_image(Path(filename): Path<String>) -> axum::response::Response {
    use axum::body::Body;
    use axum::http::{StatusCode, header};

    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Invalid filename"))
            .expect("serve_image response builder");
    }

    let claw_dir = match i_rs_claw_core::utils::claw_dir() {
        Some(d) => d,
        None => {
            return axum::response::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Cannot resolve claw directory"))
                .expect("serve_image response builder");
        }
    };

    let filepath = claw_dir.join("images").join(&filename);

    match std::fs::read(&filepath) {
        Ok(content) => {
            let mime = mime_guess::from_path(&filepath).first_or_octet_stream();
            axum::response::Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content))
                .expect("serve_image response builder")
        }
        Err(_) => axum::response::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Image not found"))
            .expect("serve_image response builder"),
    }
}
