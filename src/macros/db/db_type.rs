#[macro_export]
#[cfg(feature = "any")]
macro_rules! db_pool {
    () => {
        ::sqlx::AnyPool
    };
}

#[macro_export]
#[cfg(all(
    feature = "postgres",
    not(any(feature = "sqlite", feature = "mysql", feature = "any"))
))]
macro_rules! db_pool {
    () => {
        ::sqlx::PgPool
    };
}

#[macro_export]
#[cfg(all(
    feature = "mysql",
    not(any(feature = "sqlite", feature = "any", feature = "postgres"))
))]
macro_rules! db_pool {
    () => {
        ::sqlx::MySqlPool
    };
}

#[macro_export]
#[cfg(all(
    feature = "sqlite",
    not(any(feature = "any", feature = "mysql", feature = "postgres"))
))]
macro_rules! db_pool {
    () => {
        ::sqlx::SqlitePool
    };
}
