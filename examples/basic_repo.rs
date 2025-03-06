#![allow(dead_code)]

use sqlx::QueryBuilder;
use sqlx_utils::prelude::*;
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

repository! {
    !zst
    pub UserRepo2<User>;
}

repository_insert! {
    UserRepo<User>;

    insert_query(model) {
        sqlx::query("INSERT INTO users (name) VALUES (?)").bind(&model.name)
    }
}

repository_update! {
    UserRepo<User>;

    update_query(model) {
        sqlx::query("UPDATE users SET name = ? where id = ?").bind(model.id).bind(&model.name)
    }
}

repository_delete! {
    UserRepo<User>;

    delete_by_id_query(id) {
        sqlx::query("DELETE FROM users WHERE id = ?").bind(id)
    }

    delete_by_filter_query(filter) {
        let mut builder = QueryBuilder::new("DELETE FROM users WHERE ");

        filter.apply_filter(&mut builder);

        builder
    }
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
            .connect(&DATABASE_URL)
            .await
            .expect("Failed to connect to database"),
    );

    let user = User {
        id: 1,
        name: String::new(),
    };

    USER_REPO.save(&user).await.unwrap();
}
