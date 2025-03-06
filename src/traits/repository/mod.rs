//! [`Repository`] Trait to define a database repository

mod_def! {
    pub mod insert;
    pub mod update;
    pub mod save;
    pub mod select;
    pub mod delete;
}

use crate::traits::model::Model;
use tracing::{debug_span, Span};
use crate::mod_def;
use crate::prelude::Pool;

/// A trait that provides a standardized interface for database operations, implementing the Repository pattern
/// for PostgreSQL databases. This trait serves as a foundation for all repository implementations in the system,
/// offering both basic CRUD operations and advanced batch processing capabilities.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository manages. Must implement the [`Model`] trait.
///
/// # Design Philosophy
///
/// This trait follows several key design principles:
///
/// 1. **Separation of Concerns**: The trait separates query definition from execution, allowing for flexible
///    query construction and testing.
/// 2. **Batch Processing**: Provides optimized batch operations for better performance when dealing with multiple records.
/// 3. **Smart Defaults**: Implements higher-level operations (like [`save_all`](Repository::save_all)) in terms of simpler operations,
///    while allowing repositories to override these implementations if needed.
/// 4. **Async-first**: All operations are asynchronous, optimized for modern database interactions.
///
/// # Core Methods
///
/// Repositories must implement these fundamental methods:
///
/// ```no_run
/// fn pool(&self) -> &PgPool;               // Access to database connection
/// fn insert_one(model: &M) -> Query<'_>; // Single item insertion query
/// fn update_one(model: &M) -> Query<'_>; // Single item update query
/// fn delete_one_by_id(id: &M::Id) -> Query<'_>; // Single item deletion query
/// ```
///
/// # Optional Methods
///
/// These methods have default implementations but can be overridden:
///
/// ```no_run
/// async fn get_all(&self) -> Result<Vec<M>>                  // Retrieve all records
/// async fn get_by_id(&self, id: impl Into<M::Id>) -> Result<Option<M>> // Get by ID
/// ```
///
/// # Batch Operations
///
/// The trait provides several batch operation methods that are automatically implemented:
///
/// * [`insert_many`](Repository::insert_many)/[`insert_batch`](Repository::insert_batch): Bulk insert operations
/// * [`update_many`](Repository::update_many)/[`update_batch`](Repository::update_batch): Bulk update operations
/// * [`delete_many`](Repository::delete_many)/[`delete_batch`](Repository::delete_batch): Bulk delete operations
/// * [`save_all`](Repository::save_all)/[`save_batch`](Repository::save_batch): Smart bulk save operations that handle both inserts and updates
///
/// Each operation comes in two variants:
/// - A convenience method using the default batch size
/// - A size-parameterized version allowing custom batch sizes
///
/// # Smart Save Operations
///
/// The trait implements intelligent save operations that automatically determine whether to insert or update:
///
/// * [`save`](Repository::save): For single models - inserts if the model has no ID, updates if it does
/// * [`save_all`](Repository::save_all)/[`save_batch`](Repository::save_batch): For multiple models - efficiently batches inserts and updates separately
///
/// # Usage Example
///
/// ```rust
/// use sqlx::PgPool;
///
/// struct UserRepository {
///     pool: PgPool
/// }
///
/// impl Repository<User> for UserRepository {
///     fn pool(&self) -> &PgPool {
///         &self.pool
///     }
///
///     fn insert_one(user: &User) -> Query<'_> {
///         sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
///             .bind(&user.name)
///             .bind(&user.email)
///     }
///
///     // ... implement other required methods
/// }
///
/// // Using the repository
/// async fn create_users(repo: &UserRepository, users: Vec<User>) -> crate::Result<()> {
///     // This will automatically batch the inserts for optimal performance
///     repo.insert_many(users).await
/// }
/// ```
///
/// # Performance Considerations
///
/// 1. **Batch Processing**: The trait uses the [`BatchOperator`] to process items in chunks,
///    preventing memory overflow and maintaining optimal database performance.
///
/// 2. **Transaction Management**: Batch operations are executed within transactions to ensure
///    data consistency.
///
/// 3. **Concurrent Operations**: Where possible (like in [`save_batch`](Repository::save_batch)), independent operations
///    are executed concurrently using [`try_join!`](futures::try_join).
///
/// # Instrumentation
///
/// All public methods are instrumented with tracing at the debug level, facilitating
/// debugging and performance monitoring. The `skip_all` directive prevents sensitive
/// data from being logged.
///
/// # Error Handling
///
/// All operations return [`crate::Result<T>`], providing consistent error handling across
/// the application. This includes:
/// - Database errors
/// - Validation errors
/// - Transaction errors
///
/// # Implementation Notes
///
/// 1. The trait leverages generics and associated types to maintain type safety while
///    providing flexibility.
///
/// 2. Default implementations of batch operations use the [`DEFAULT_BATCH_SIZE`] constant,
///    but custom sizes can be specified using the `_batch` variants.
///
/// 3. The [`save_batch`](Repository::save_batch) implementation intelligently sorts models into insert and update
///    operations, executing them in the most efficient way possible.
///
/// 4. Unimplemented methods ([`get_all`](Repository::get_all) and [`get_by_id`](Repository::get_by_id)) provide clear error messages
///    when called without implementation.
pub trait Repository<M>
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

    #[inline]
    fn repository_span() -> Span {
        debug_span!("Repository")
    }
}
