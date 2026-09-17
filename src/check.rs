use serde_json::{json,Value};
use axum::Json;

pub fn print_startup_info(port: u16) {
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                 📦  GEARLY PRODUCT API                   ║");
    println!("║           Car Parts Marketplace Product Backend          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  ✓ Database       Connected");
    println!("  ✓ JWT            Initialized");
    println!("  ✓ Server         Ready");
    println!();
    println!("  Routes");
    println!("  ────────────────────────────────────────────────────────");
    println!("  POST   /create_product");
    println!("  GET    /get_products");
    println!("  GET    /get_product/:id");
    println!("  PUT    /update_product/:id");
    println!("  POST   /delete_product");
    println!("  GET    /health");
    println!();
    println!("  🚀 Server running at http://127.0.0.1:{}", port);
    println!();
}

#[axum::debug_handler]
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ready"
    }
    ))
}