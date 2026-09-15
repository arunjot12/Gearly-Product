use axum::{
    Router, middleware,
    routing::{get, post, put},
    serve,
};
use tokio::net::TcpListener;
pub mod auth;
pub mod db;
pub mod model;
pub mod product;
pub mod schema;
use crate::{
    auth::{auth::JwtService, middleware::auth_middleware},
    db::{DbPool, create_pool},
    product::api::{create_part, delete_product, get_product, get_products, update_product},
};

#[derive(Clone)]
pub struct AppState {
    db_pool: DbPool,
    jwt_service: JwtService,
}

#[tokio::main]
async fn main() {
    let jwt = std::env::var("JwtService").expect("JWT secret needs to set");

    let jwt_service = JwtService::new(&jwt);
    let pool = create_pool();

    let state = AppState {
        db_pool: pool,
        jwt_service: jwt_service,
    };
    let app = Router::new()
        .route("/create_product", post(create_part))
        .route("/get_product", get(get_product))
        .route("/get_products", get(get_products))
        .route("/get_product/:id", get(get_product))
        .route("/delete_product", post(delete_product))
        .route("/update_product/:id", put(update_product))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap_or_else(|e| panic!("failed to bind to port {port}: {e}"));

    serve(listener, app).await.unwrap();
}
