use tower_http::cors::{Any,CorsLayer};
use axum::http::HeaderValue;

pub fn cors_allow() -> CorsLayer
{
 CorsLayer::new()
    .allow_origin(
        "https://gearly-frontend.vercel.app"
            .parse::<HeaderValue>()
            .unwrap()
    )
    .allow_methods(Any)
    .allow_headers(Any)
}