use super::Database;
use sqlx::Database as DatabaseTrait;

#[cfg(feature = "any")]
pub type Query<'a> = sqlx::query::Query<'a, sqlx::Any, sqlx::any::AnyArguments<'a>>;

#[cfg(all(
    feature = "postgres",
    not(any(feature = "sqlite", feature = "mysql", feature = "any"))
))]
pub type Query<'a> = sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>;

#[cfg(all(
    feature = "mysql",
    not(any(feature = "sqlite", feature = "any", feature = "postgres"))
))]
pub type Query<'a> = sqlx::query::Query<'a, sqlx::MySql, sqlx::mysql::MySqlArguments>;

#[cfg(all(
    feature = "sqlite",
    not(any(feature = "any", feature = "mysql", feature = "postgres"))
))]
pub type Query<'a> = sqlx::query::Query<'a, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'a>>;

/// A single SQL query as a prepared statement, mapping results using [`FromRow`](sqlx::FromRow).
/// Returned by [`query_as()`](sqlx::query_as()) or [`query_as()`](sqlx::query_as!). This is a wrapper [`QueryAs`](sqlx::query::QueryAs) abstracting away
/// the database into a simpler format using generic `DB` which implements [`Database`](DatabaseTrait)
pub type QueryAs<'q, T, DB = Database> =
    sqlx::query::QueryAs<'q, Database, T, <DB as DatabaseTrait>::Arguments<'q>>;

pub type QueryBuilder<'args, DB = Database> = sqlx::QueryBuilder<'args, DB>;
