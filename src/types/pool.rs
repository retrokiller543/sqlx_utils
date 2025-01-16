use super::db_type;

db_type! {
    pub type Pool = [sqlx::AnyPool, sqlx::postgres::PgPool, sqlx::mysql::MySqlPool, sqlx::sqlite::SqlitePool]
}

db_type! {
    pub type PoolOptions = [sqlx::any::AnyPoolOptions, sqlx::postgres::PgPoolOptions, sqlx::mysql::MySqlPoolOptions, sqlx::sqlite::SqlitePoolOptions]
}