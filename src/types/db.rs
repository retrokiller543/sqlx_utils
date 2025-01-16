use super::db_type;

db_type! {
    pub type Database = [sqlx::Any, sqlx::postgres::Postgres, sqlx::mysql::MySql, sqlx::sqlite::Sqlite]
}