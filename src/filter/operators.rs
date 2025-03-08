use crate::{sql_delimiter, sql_operator};

use crate::traits::SqlFilter;
use sqlx::QueryBuilder;

/// UNSAFE AF!!!!
pub struct Raw(pub &'static str);

impl<'args> SqlFilter<'args> for Raw {
    #[inline]
    #[cfg(feature = "any")]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Any>) {
        if self.should_apply_filter() {
            builder.push(self.0);
        }
    }
    #[inline]
    #[cfg(all(feature = "postgres", not(any(feature = "sqlite", feature = "mysql", feature = "any"))))]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Postgres>) {
        if self.should_apply_filter() {
            builder.push(self.0);
        }
    }
    #[inline]
    #[cfg(all(feature = "mysql", not(any(feature = "sqlite", feature = "any", feature = "postgres"))))]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::MySql>) {
        if self.should_apply_filter() {
            builder.push(self.0);
        }
    }
    #[inline]
    #[cfg(all(feature = "sqlite", not(any(feature = "any", feature = "mysql", feature = "postgres"))))]
    fn apply_filter(self, builder: &mut QueryBuilder<'args, sqlx::Sqlite>) {
        if self.should_apply_filter() {
            builder.push(self.0);
        }
    }

    #[inline]
    fn should_apply_filter(&self) -> bool {
        !self.0.is_empty()
    }
}


sql_delimiter! {
    pub struct And<L, R> {
        pub left: L,
        pub right: R
    }

    apply_filter(s, builder) {
        match (
            s.left.should_apply_filter(),
            s.right.should_apply_filter(),
        ) {
            (true, true) => {
                s.left.apply_filter(builder);
                builder.push(" AND ");
                s.right.apply_filter(builder);
            }
            (true, false) => {
                s.left.apply_filter(builder);
            }
            (false, true) => {
                s.right.apply_filter(builder);
            }
            (false, false) => {}
        }
    }

    should_apply_filter(s) {
        s.left.should_apply_filter() || s.right.should_apply_filter()
    }
}

sql_delimiter! {
    pub struct Or<L, R> {
        pub left: L,
        pub right: R
    }

    apply_filter(s, builder) {
        match (
            s.left.should_apply_filter(),
            s.right.should_apply_filter(),
        ) {
            (true, true) => {
                s.left.apply_filter(builder);
                builder.push(" OR ");
                s.right.apply_filter(builder);
            }
            (true, false) => {
                s.left.apply_filter(builder);
            }
            (false, true) => {
                s.right.apply_filter(builder);
            }
            (false, false) => {}
        }
    }

    should_apply_filter(s) {
        s.left.should_apply_filter() || s.right.should_apply_filter()
    }
}

sql_delimiter! {
    pub struct Not<T> {
        pub inner: T,
    }

    apply_filter(s, builder) {
        if s.should_apply_filter() {
            builder.push("NOT ");
            s.inner.apply_filter(builder);
        }
    }

    should_apply_filter(s) {
        s.inner.should_apply_filter()
    }
}

sql_operator!(pub Equals, "=");
sql_operator!(pub Like<String>, "LIKE");
sql_operator!(pub InValues[], "IN");
sql_operator!(pub NotEquals, "!=");
sql_operator!(pub GreaterThan, ">");
sql_operator!(pub LessThan, "<");
sql_operator!(pub GreaterThanOrEqual, ">=");
sql_operator!(pub LessThanOrEqual, "<=");
sql_operator!(pub ILike<String>, "ILIKE");
sql_operator!(pub NotInValues[], "NOT IN");
sql_operator!(pub NoOpFilter);