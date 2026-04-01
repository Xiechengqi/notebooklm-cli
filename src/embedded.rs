use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use include_dir::{Dir, include_dir};

static FRONTEND: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/out");

pub async fn serve_static(req: Request) -> Response {
    let raw = req.uri().path();

    // Normalize: strip leading slash, map "/" to "index.html"
    let path = raw.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    // Try exact match first
    if let Some(file) = FRONTEND.get_file(path) {
        return serve_file(file);
    }

    // Try <path>/index.html for client-side routing (Next.js static export)
    let html_path = format!("{}/index.html", path.trim_end_matches('/'));
    if let Some(file) = FRONTEND.get_file(html_path.as_str()) {
        return serve_file(file);
    }

    // Try <path>.html
    let html_path2 = format!("{}.html", path);
    if let Some(file) = FRONTEND.get_file(html_path2.as_str()) {
        return serve_file(file);
    }

    // SPA fallback: serve root index.html for unknown paths
    if let Some(file) = FRONTEND.get_file("index.html") {
        return serve_file(file);
    }

    (StatusCode::NOT_FOUND, "not found").into_response()
}

fn serve_file(file: &include_dir::File<'_>) -> Response {
    let mime = mime_for_path(file.path().to_str().unwrap_or(""));
    (
        [(header::CONTENT_TYPE, mime)],
        file.contents(),
    )
        .into_response()
}

fn mime_for_path(path: &str) -> &'static str {
    if path.ends_with(".html") { "text/html; charset=utf-8" }
    else if path.ends_with(".css") { "text/css; charset=utf-8" }
    else if path.ends_with(".js") || path.ends_with(".mjs") { "application/javascript; charset=utf-8" }
    else if path.ends_with(".json") { "application/json" }
    else if path.ends_with(".svg") { "image/svg+xml" }
    else if path.ends_with(".png") { "image/png" }
    else if path.ends_with(".jpg") || path.ends_with(".jpeg") { "image/jpeg" }
    else if path.ends_with(".ico") { "image/x-icon" }
    else if path.ends_with(".woff2") { "font/woff2" }
    else if path.ends_with(".woff") { "font/woff" }
    else if path.ends_with(".ttf") { "font/ttf" }
    else if path.ends_with(".txt") { "text/plain; charset=utf-8" }
    else if path.ends_with(".xml") { "application/xml" }
    else { "application/octet-stream" }
}
