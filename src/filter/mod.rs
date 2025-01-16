mod_def! {
    pub mod operators;
}

use crate::mod_def;
use crate::traits::SqlFilter;
use sqlx::QueryBuilder;

pub struct Filter<T>(T);

impl<T> Filter<T> {
    #[inline]
    pub fn new(filter: T) -> Self {
        Filter(filter)
    }
}

#[allow(clippy::should_implement_trait)]
impl<'args, T: 'args> Filter<T>
where
    T: SqlFilter<'args>,
{
    #[inline]
    pub fn and<U>(self, other: U) -> Filter<And<T, U>>
    where
        U: SqlFilter<'args>,
    {
        Filter(And {
            left: self.0,
            right: other,
        })
    }

    #[inline]
    pub fn or<U>(self, other: U) -> Filter<Or<T, U>>
    where
        U: SqlFilter<'args>,
    {
        Filter(Or {
            left: self.0,
            right: other,
        })
    }

    #[inline]
    pub fn not(self) -> Filter<Not<T>> {
        Filter(Not::new(self.0))
    }
}

impl<'args, T> SqlFilter<'args> for Filter<T>
where
    T: SqlFilter<'args>,
{
    #[inline]
    #[cfg(feature = "any")]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Any>) {
        self.0.apply_filter(builder);
    }
    #[inline]
    #[cfg(all(
        feature = "postgres",
        not(any(feature = "sqlite", feature = "mysql", feature = "any"))
    ))]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Postgres>) {
        self.0.apply_filter(builder);
    }
    #[inline]
    #[cfg(all(
        feature = "mysql",
        not(any(feature = "sqlite", feature = "any", feature = "postgres"))
    ))]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::MySql>) {
        self.0.apply_filter(builder);
    }
    #[inline]
    #[cfg(all(
        feature = "sqlite",
        not(any(feature = "any", feature = "mysql", feature = "postgres"))
    ))]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Sqlite>) {
        self.0.apply_filter(builder);
    }

    #[inline]
    fn should_apply_filter(&self) -> bool {
        self.0.should_apply_filter()
    }
}
