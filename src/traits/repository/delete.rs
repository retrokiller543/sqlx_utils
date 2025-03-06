//! Trait for adding delete capabilities to a repository

use crate::traits::{Model, Repository};
use crate::types::Query;
use crate::utils::{BatchOperator, DEFAULT_BATCH_SIZE};

/// Trait for repositories that can delete records from the database.
///
/// The `DeleteRepository` trait extends the base [`Repository`] trait with methods
/// for deleting records. It provides standardized ways to delete both individual
/// records and batches of records, optimizing database interactions while maintaining
/// data integrity.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository deletes. Must implement the [`Model`] trait.
///
/// # Examples
///
/// Basic implementation:
/// ```rust
/// # use sqlx_utils::traits::{Model, Repository, DeleteRepository};
/// # use sqlx_utils::types::{Pool, Query};
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
/// impl DeleteRepository<User> for UserRepository {
///     fn delete_query_by_id(id: &i32) -> Query<'_> {
///         sqlx::query("DELETE FROM users WHERE id = $1")
///             .bind(id)
///     }
/// }
///
/// // Usage
/// # async fn example(repo: &UserRepository) -> sqlx_utils::Result<()> {
/// // Delete a single user
/// repo.delete_by_id(1).await?;
///
/// // Delete multiple users
/// let ids = vec![1, 2, 3];
/// repo.delete_many(ids).await?;
/// # Ok(())
/// # }
/// ```
///
/// Using the macro for simpler implementation:
/// ```rust
/// # use sqlx_utils::{repository, repository_delete};
/// # use sqlx_utils::traits::Model;
/// # use sqlx_utils::types::Query;
/// # struct User { id: i32, name: String }
/// # impl Model for User {
/// #     type Id = i32;
/// #     fn get_id(&self) -> Option<Self::Id> { Some(self.id) }
/// # }
///
/// repository! {
///     UserRepository<User>;
///
///     // Implementation will go here
/// }
///
/// repository_delete! {
///     UserRepository<User>;
///
///     delete_query_by_id(id) {
///         sqlx::query("DELETE FROM users WHERE id = $1")
///             .bind(id)
///     }
/// }
/// ```
///
/// # Implementation Notes
///
/// 1. Required method: [`delete_query_by_id`](DeleteRepository::delete_query_by_id) - Defines how to create a deletion query for a model ID
/// 2. Provided methods:
///    - [`delete_by_id`](DeleteRepository::delete_by_id) - Deletes a single record by ID
///    - [`delete_many`](DeleteRepository::delete_many) - Deletes multiple records using the default batch size
///    - [`delete_batch`](DeleteRepository::delete_batch) - Deletes multiple records with a custom batch size
/// 3. All batch operations use transactions to ensure data consistency
/// 4. Performance is optimized through batching and connection pooling
/// 5. Consider implementing soft deletes if required by your application
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `DeleteRepository<{M}>` trait",
    label = "this type does not implement `DeleteRepository` for model type `{M}`",
    message = "`{Self}` must implement `DeleteRepository<{M}>` to delete `{M}` records"
)]
pub trait DeleteRepository<M: Model>: Repository<M> {
    /// Creates a SQL query to delete a record by its ID.
    ///
    /// This method generates a DELETE statement that will remove exactly one record
    /// from the database based on its primary key. It's designed to be both safe
    /// and efficient by using parameterized queries.
    ///
    /// # Parameters
    ///
    /// * `id` - A reference to the ID of the record to delete
    ///
    /// # Returns
    ///
    /// * [`Query`] - A prepared PostgreSQL DELETE query
    ///
    /// # Implementation Notes
    ///
    /// Consider:
    /// 1. Handling soft deletes if required
    /// 2. Checking foreign key constraints
    /// 3. Implementing cascading deletes if needed
    fn delete_query_by_id(id: &M::Id) -> Query<'_>;

    /// Removes a single record from the database by its identifier.
    ///
    /// This method executes the deletion query generated by [`delete_query_by_id`](Self::delete_query_by_id). It provides
    /// a simple interface for removing individual records while handling all necessary database
    /// interactions and error management.
    ///
    /// # Parameters
    ///
    /// * `id` - Any value that can be converted into the model's ID type
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if the deletion was executed, or an error if the operation failed
    ///
    /// # Example
    ///
    /// ```no_compile
    /// async fn remove_user(repo: &UserRepository, user_id: i32) -> crate::Result<()> {
    ///     repo.delete_by_id(user_id).await
    /// }
    /// ```
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn delete_by_id(&self, id: impl Into<M::Id>) -> crate::Result<()> {
        Self::delete_query_by_id(&id.into())
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Deletes multiple records using the default batch size.
    ///
    /// This is a convenience wrapper around [`delete_batch`](Self::delete_batch) that uses [`DEFAULT_BATCH_SIZE`].
    /// It provides a simpler interface for bulk deletions when the default batch size
    /// is appropriate.
    ///
    /// # Parameters
    ///
    /// * `ids` - An iterator yielding IDs of records to delete
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if all deletions were executed, or an error if any operation failed
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn delete_many(&self, ids: impl IntoIterator<Item = M::Id>) -> crate::Result<()> {
        <Self as DeleteRepository<M>>::delete_batch::<DEFAULT_BATCH_SIZE>(self, ids).await
    }

    /// Performs a batched deletion operation with a specified batch size.
    ///
    /// Similar to other batch operations, this method uses [`BatchOperator`] to efficiently
    /// process large numbers of deletions in chunks, maintaining optimal performance
    /// and preventing resource exhaustion.
    ///
    /// # Type Parameters
    ///
    /// * `N` - The size of each batch to process
    ///
    /// # Parameters
    ///
    /// * `ids` - An iterator yielding IDs of records to delete
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if all batches were processed, or an error if any operation failed
    ///
    /// # Implementation Details
    ///
    /// The method:
    /// 1. Chunks the input IDs into batches of size N
    /// 2. Processes each batch in a transaction using [`delete_query_by_id`](Self::delete_query_by_id)
    /// 3. Maintains ACID properties within each batch
    ///
    /// # Performance Considerations
    ///
    /// Consider batch size carefully:
    /// - Too small: More overhead from multiple transactions
    /// - Too large: Higher memory usage and longer transaction times
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn delete_batch<const N: usize>(
        &self,
        ids: impl IntoIterator<Item = M::Id>,
    ) -> crate::Result<()> {
        BatchOperator::<M::Id, N>::execute_query(ids, self.pool(), Self::delete_query_by_id).await
    }
}