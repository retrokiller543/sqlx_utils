//! Sql filtering trait for defining type safe dynamic filters.

use crate::types::Database;
use sqlx::{Database as DatabaseTrait, QueryBuilder};

/// Trait for creating SQL filter conditions that can be applied to database queries.
///
/// The `SqlFilter` trait provides a standardized way to define reusable, type-safe
/// SQL filter conditions. Filters can be composed and combined using logical operators
/// to create complex query criteria.
///
/// # Type Parameters
///
/// * `'args`: The lifetime for query arguments
/// * `DB`: The database backend type (defaults to the configured default database)
///
/// # Examples
///
/// Basic implementation using the macro:
/// ```rust
/// # use sqlx_utils::sql_filter;
/// # use sqlx_utils::traits::SqlFilter;
/// # use sqlx_utils::types::Database;
/// # use sqlx::QueryBuilder;
///
/// sql_filter! {
///     pub struct UserFilter {
///         SELECT * FROM users WHERE
///         id = i32 AND
///         ?name LIKE String AND
///         ?email LIKE String
///     }
/// }
///
/// // Usage
/// # fn example() {
/// let filter = UserFilter::new(42)
///     .name("Alice%")
///     .email("alice@example.com");
///
/// let mut builder = QueryBuilder::<Database>::new("SELECT * FROM users WHERE ");
/// filter.apply_filter(&mut builder);
/// # }
/// ```
///
/// Custom implementation:
/// ```rust
/// # use sqlx_utils::traits::SqlFilter;
/// # use sqlx_utils::types::Database;
/// # use sqlx::QueryBuilder;
///
/// struct AgeRangeFilter {
///     min_age: Option<i32>,
///     max_age: Option<i32>,
/// }
///
/// impl<'args> SqlFilter<'args, Database> for AgeRangeFilter {
///     fn apply_filter(self, builder: &mut QueryBuilder<'args, Database>) {
///         if self.min_age.is_some() || self.max_age.is_some() {
///             let mut first = true;
///
///             if let Some(min) = self.min_age {
///                 if !first { builder.push(" AND "); }
///                 builder.push("age >= ");
///                 builder.push_bind(min);
///                 first = false;
///             }
///
///             if let Some(max) = self.max_age {
///                 if !first { builder.push(" AND "); }
///                 builder.push("age <= ");
///                 builder.push_bind(max);
///             }
///         }
///     }
///
///     fn should_apply_filter(&self) -> bool {
///         self.min_age.is_some() || self.max_age.is_some()
///     }
/// }
/// ```
///
/// # Implementation Notes
///
/// When implementing this trait:
///
/// 1. The [`apply_filter`](SqlFilter::apply_filter) method should add SQL conditions to the builder
/// 2. The [`should_apply_filter`](SqlFilter::should_apply_filter) method should return `true` if this filter has criteria to apply
/// 3. Consider using the [`sql_filter!`](crate::sql_filter) macro for common filter patterns
/// 4. Ensure proper parameterization to prevent SQL injection
#[diagnostic::on_unimplemented(
    message = "The filter type `{Self}` must implement `SqlFilter<'args>` to be used in queries",
    label = "this type does not implement `SqlFilter<'args>`",
    note = "Type `{Self}` does not implement the `SqlFilter` trait with the required lifetime `'args`.",
    note = "SqlFilter is implemented by default for the `sqlx_utils::types::Database` database, things might not work if you are using a custom database."
)]
pub trait SqlFilter<'args, DB: DatabaseTrait = Database> {
    /// Applies this filter's conditions to a SQL query builder.
    ///
    /// This method should add the necessary SQL conditions represented by this filter
    /// to the provided query builder. It should handle binding parameters securely.
    ///
    /// # Parameters
    ///
    /// * `self` - The filter instance, consumed during application
    /// * `builder` - The query builder to which the filter will be applied
    fn apply_filter(self, builder: &mut QueryBuilder<'args, DB>);

    /// Determines whether this filter has meaningful conditions to apply.
    ///
    /// This method should return `true` if the filter has non-default conditions
    /// that should be included in a query, or `false` if the filter is empty or
    /// represents default criteria that don't need to be applied.
    ///
    /// # Returns
    ///
    /// * `true` - If the filter has conditions to apply
    /// * `false` - If the filter has no conditions to apply
    fn should_apply_filter(&self) -> bool;
}
