#![allow(dead_code)]

use sqlx::any::install_default_drivers;
use sqlx_utils::pool::initialize_db_pool;
use sqlx_utils::repository;
use sqlx_utils::traits::{Model, Repository};
use sqlx_utils::types::PoolOptions;
use sqlx_utils_macro::sql_filter;
use std::sync::LazyLock;
use std::time::Duration;

pub static DATABASE_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("DATABASE_URL").expect("failed to get env var"));

sql_filter! {
    pub struct UserFilter {
        SELECT * FROM users WHERE ?id = i64
    }
}

struct User {
    id: i64,
    name: String,
}

impl Model for User {
    type Id = i64;

    fn get_id(&self) -> Option<i64> {
        Some(self.id)
    }
}

repository! {
    pub UserRepo<User>;
}

#[tokio::main]
async fn main() {
    install_default_drivers();

    initialize_db_pool(
        PoolOptions::new()
            .max_connections(21)
            .min_connections(5)
            .idle_timeout(Duration::from_secs(60 * 10))
            .max_lifetime(Duration::from_secs(60 * 60 * 24))
            .acquire_timeout(Duration::from_secs(20))
            .connect("postgresql://postgres:root@localhost:5432/tosic_db")
            .await
            .expect("Failed to connect to database"),
    );

    let user = User {
        id: 1,
        name: String::new(),
    };

    USER_REPO.save(&user).await.unwrap();
}
