//! [`Repository`] Trait to define a database repository

mod_def! {
    pub mod insert;
    pub mod update;
    pub mod save;
    pub mod select;
    pub mod delete;
    pub mod transaction;
}

use crate::mod_def;
use crate::prelude::Pool;
use crate::traits::model::Model;
use tracing::{debug_span, Span};

/// A trait that provides a standardized interface for database operations, implementing the Repository pattern.
///
/// This trait serves as a foundation for all repository implementations in the system,
/// offering both basic access to database connections and advanced batch processing capabilities.
/// It centralizes the logic for database access patterns and promotes consistent error handling
/// across the application.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository manages. Must implement the [`Model`] trait.
///
/// # Design Philosophy
///
/// This trait follows several key design principles:
///
/// 1. **Separation of Concerns**: The repository isolates database access logic from business logic
/// 2. **Type Safety**: Uses generics and associated types to maintain compile-time type checking
/// 3. **Instrumentation Support**: Built-in tracing for debugging and monitoring
/// 4. **Extensibility**: Serves as a base for more specialized repository traits
///
/// # Core Methods
///
/// Repositories must implement:
///
/// ```no_compile
/// fn pool(&self) -> &Pool;  // Access to database connection pool
/// ```
///
/// # Usage Example
///
/// Basic repository implementation:
/// ```rust
/// # use sqlx_utils::prelude::*;
/// # use sqlx_utils::types::Pool;
/// # struct User { id: i32, name: String }
/// # impl Model for User {
/// #     type Id = i32;
/// #     fn get_id(&self) -> Option<Self::Id> { Some(self.id) }
/// # }
///
/// struct UserRepository {
///     pool: Pool
/// }
///
/// impl Repository<User> for UserRepository {
///     fn pool(&self) -> &Pool {
///         &self.pool
///     }
/// }
///
/// // Adding specialized capabilities through extension traits
/// impl InsertableRepository<User> for UserRepository {
///     fn insert_query(user: &User) -> Query<'_> {
///         sqlx::query("INSERT INTO users (name) VALUES ($1)")
///             .bind(&user.name)
///     }
/// }
/// ```
///
/// # Error Handling
///
/// Repository methods return [`crate::Result<T>`], providing consistent error handling across
/// the application. This includes database errors, validation errors, and transaction errors.
///
/// # Implementation Notes
///
/// 1. The trait is intended to be extended by more specialized traits that provide
///    concrete CRUD operations (like [`InsertableRepository`], [`UpdatableRepository`], etc.)
/// 2. The static `repository_span()` method provides consistent tracing instrumentation
///    across all repositories
/// 3. Use the included macros ([`repository!`](crate::repository), [`repository_insert!`](crate::repository_insert), etc.) to reduce
///    boilerplate when implementing repositories
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `Repository<{M}>` trait",
    label = "this type does not implement `Repository` for model type `{M}`",
    message = "`{Self}` must implement `Repository<{M}>` to provide database operations for `{M}`"
)]
pub trait Repository<M>: Sync
where
    M: Model,
{
    /// Gets a reference to the database connection pool used by this repository.
    ///
    /// The pool is a fundamental component that manages database connections efficiently,
    /// handling connection pooling, timeouts, and reconnection strategies. Each repository
    /// instance maintains its own reference to a pool, but multiple repositories can share
    /// the same underlying pool to optimize resource usage.
    ///
    /// # Returns
    ///
    /// * `&`[`Pool`] - A reference to the Database connection pool
    fn pool(&self) -> &Pool;

    /// Creates a tracing span for repository operations.
    ///
    /// This method provides a consistent way to create spans for tracing and
    /// debugging repository operations. All repository methods should use this
    /// span as their parent span to ensure proper hierarchical tracing.
    ///
    /// # Returns
    ///
    /// * [`Span`] - A tracing span for repository operations
    #[inline]
    fn repository_span() -> Span {
        debug_span!("Repository")
    }
}
