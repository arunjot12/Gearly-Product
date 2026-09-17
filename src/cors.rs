use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;

pub fn cors_allow() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:5173"))
        .allow_methods(Any)
        .allow_headers(Any)
}