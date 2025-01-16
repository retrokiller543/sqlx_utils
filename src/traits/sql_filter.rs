use sqlx::QueryBuilder;

#[cfg(feature = "any")]
pub trait SqlFilter<'args> {
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Any>);
    fn should_apply_filter(&self) -> bool;
}

#[cfg(all(feature = "postgres", not(any(feature = "sqlite", feature = "mysql", feature = "any"))))]
pub trait SqlFilter<'args> {
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Postgres>);
    fn should_apply_filter(&self) -> bool;
}

#[cfg(all(feature = "mysql", not(any(feature = "sqlite", feature = "any", feature = "postgres"))))]
pub trait SqlFilter<'args> {
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::MySql>);
    fn should_apply_filter(&self) -> bool;
}

#[cfg(all(feature = "sqlite", not(any(feature = "any", feature = "mysql", feature = "postgres"))))]
pub trait SqlFilter<'args> {
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Sqlite>);
    fn should_apply_filter(&self) -> bool;
}
