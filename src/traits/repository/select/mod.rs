//! Trait for adding select capabilities to a repository

mod_def! {
    pub mod filter;
}

use crate::mod_def;
use crate::traits::{Model, Repository};

/// Trait for repositories that can retrieve records from the database.
///
/// The `SelectRepository` trait extends the base [`Repository`] trait with methods
/// for querying and retrieving records. It defines a standard interface for fetching
/// both individual records by ID and collections of records.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository retrieves. Must implement the [`Model`] trait.
///
/// # Examples
///
/// Basic implementation:
/// ```rust
/// # use sqlx_utils::traits::{Model, Repository, SelectRepository};
/// # use sqlx_utils::types::Pool;
/// # #[derive(sqlx::FromRow)]
/// # struct User { id: i32, name: String }
/// # impl Model for User {
/// #     type Id = i32;
/// #     fn get_id(&self) -> Option<Self::Id> { Some(self.id) }
/// # }
/// # struct UserRepository { pool: Pool }
/// # impl Repository<User> for UserRepository {
/// #     fn pool(&self) -> &Pool { &self.pool }
/// # }
///
/// impl SelectRepository<User> for UserRepository {
///     async fn get_all(&self) -> sqlx_utils::Result<Vec<User>> {
///         sqlx::query_as("SELECT * FROM users")
///             .fetch_all(self.pool())
///             .await
///             .map_err(Into::into)
///     }
///
///     async fn get_by_id(&self, id: impl Into<i32>) -> sqlx_utils::Result<Option<User>> {
///         let id = id.into();
///         sqlx::query_as("SELECT * FROM users WHERE id = $1")
///             .bind(id)
///             .fetch_optional(self.pool())
///             .await
///             .map_err(Into::into)
///     }
/// }
///
/// // Usage
/// # async fn example(repo: &UserRepository) -> sqlx_utils::Result<()> {
/// // Get a single user
/// let user = repo.get_by_id(1).await?;
///
/// // Get all users
/// let all_users = repo.get_all().await?;
/// # Ok(())
/// # }
/// ```
///
/// # Implementation Notes
///
/// 1. Required methods:
///    - [`get_all`](SelectRepository::get_all) - Retrieves all records of the model type
///    - [`get_by_id`](SelectRepository::get_by_id) - Retrieves a single record by its ID
/// 2. Consider implementing pagination for [`get_all`](SelectRepository::get_all) if the table may contain a large
///    number of records
/// 3. Use parameter binding to prevent SQL injection
/// 4. Consider caching strategies for frequently accessed data
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `SelectRepository<{M}>` trait",
    label = "this type does not implement `SelectRepository` for model type `{M}`",
    message = "`{Self}` must implement `SelectRepository<{M}>` to query for `{M}` records"
)]
pub trait SelectRepository<M: Model>: Repository<M> {
    /// Retrieves all records of this model type from the database.
    ///
    /// By default, this method is unimplemented and will panic if called. Repositories
    /// should override this method when they need to support retrieving all records.
    /// Consider implementing pagination or limiting the result set size for large tables.
    ///
    /// # Returns
    ///
    /// * [`crate::Result<Vec<M>>`] - A Result containing a vector of all models if successful
    ///
    /// # Warning
    ///
    /// Be cautious with this method on large tables as it could consume significant
    /// memory and impact database performance. Consider implementing pagination instead.
    async fn get_all(&self) -> crate::Result<Vec<M>>;

    /// Retrieves a single model instance by its ID.
    ///
    /// By default, this method is unimplemented. When implemented, it should efficiently
    /// fetch exactly one record matching the provided ID. The method accepts any type
    /// that can be converted into the model's ID type for flexibility.
    ///
    /// # Parameters
    ///
    /// * `id` - Any value that can be converted into the model's ID type
    ///
    /// # Returns
    ///
    /// * [`crate::Result<Option<M>>`] - A Result containing either:
    ///   - Some(model) if a record was found
    ///   - None if no record exists with the given ID
    async fn get_by_id(&self, id: impl Into<M::Id>) -> crate::Result<Option<M>>;
}