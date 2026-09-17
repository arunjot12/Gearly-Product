use deadpool_diesel::mysql::{Manager, Pool};
use dotenv::dotenv;
use std::env;

pub type DbPool = Pool;

pub fn create_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("Database not found");
    let manager = Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);

    Pool::builder(manager)
        .max_size(10) // tune this based on your DB's max_connections and load
        .build()
        .expect("Failed to create DB pool")
}

#[derive(serde::Deserialize)]
pub struct Pagination{
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64
}

fn default_limit() -> i64{
    20
}

fn default_offset() -> i64{
    0
}