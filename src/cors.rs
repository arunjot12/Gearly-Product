use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;

pub fn cors_allow() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(HeaderValue::from_static("https://gearly-frontend.vercel.app/"))
        .allow_methods(Any)
        .allow_headers(Any)
}