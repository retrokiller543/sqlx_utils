//! Trait for adding delete capabilities to a repository

use crate::prelude::{Database, SqlFilter};
use crate::traits::{Model, Repository};
use crate::types::Query;
use crate::utils::{tracing_debug_log, BatchOperator, DEFAULT_BATCH_SIZE};
use sqlx::{Executor, QueryBuilder};

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
/// # use sqlx_utils::prelude::SqlFilter;
/// use sqlx_utils::traits::{Model, Repository, DeleteRepository};
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
///     fn delete_by_id_query(id: &i32) -> Query<'_> {
///         sqlx::query("DELETE FROM users WHERE id = $1")
///             .bind(id)
///     }
///
///     fn delete_by_filter_query<'args>(filter: impl SqlFilter<'args>) -> Query<'args> {
///         let mut builder = sqlx::query_builder::QueryBuilder::new("DELETE FROM users");
///
///         if filter.should_apply_filter() {
///             builder.push("WHERE ");
///             filter.apply_filter(&mut builder);
///         }
///
///         builder.build()
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
/// repo.delete_many_by_id(ids).await?;
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
///     // if you need to override any method other than `Repository::pool` they will go here
/// }
///
/// repository_delete! {
///     UserRepository<User>;
///
///     delete_by_id_query(id) {
///         sqlx::query("DELETE FROM users WHERE id = $1")
///             .bind(id)
///     }
/// }
/// ```
///
/// # Implementation Notes
///
/// 1. Required method: [`delete_by_id_query`](DeleteRepository::delete_by_id_query) - Defines how to create a deletion query for a model ID
/// 2. Provided methods:
///    - [`delete_by_id`](DeleteRepository::delete_by_id) - Deletes a single record by ID
///    - [`delete_many_by_id`](DeleteRepository::delete_many_by_id) - Deletes multiple records by id using the default batch size
///    - [`delete_batch_by_id`](DeleteRepository::delete_batch_by_id) - Deletes multiple records by id with a custom batch size
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
    /// * [`Query`] - A prepared SQL DELETE query
    ///
    /// # Implementation Notes
    ///
    /// Consider:
    /// 1. Handling soft deletes if required
    /// 2. Checking foreign key constraints
    /// 3. Implementing cascading deletes if needed
    fn delete_by_id_query(id: &M::Id) -> Query<'_>;

    /// Creates a SQL query to delete a record by a given [`SqlFilter`].
    ///
    /// This method generates a DELETE statement that will remove all records that match a given [`SqlFilter`]. It's designed to be both safe
    /// and efficient by using parameterized queries. The query can do a soft delete or a complete remove of it,
    /// that detail is up to the implementor, the rest of the Trait expects the query to not return anything however and the query should reflect that.
    ///
    /// # Parameters
    ///
    /// * `filter` - The filter used when generating the query.
    ///
    /// # Returns
    ///
    /// * [`Query`] - A prepared SQL query to DELETE records or soft delete them
    ///
    /// # Implementation Notes
    ///
    /// Consider:
    /// 1. Handling soft deletes if required
    /// 2. Checking foreign key constraints
    /// 3. Implementing cascading deletes if needed
    /// 4. If called via the default implementation of [`delete_by_filter_with_executor`](Self::delete_by_filter_with_executor)
    ///    the filter will be guaranteed to be applied.
    fn delete_by_filter_query<'args>(
        filter: impl SqlFilter<'args>,
    ) -> QueryBuilder<'args, Database>;

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "delete_by_id",]
        /// Removes a single record from the database by its identifier.
        ///
        /// This method executes the deletion query generated by [`delete_by_id_query`](Self::delete_by_id_query) and uses the [`Executor`] `tx` for doing it. It provides
        /// a simple interface for removing individual records while handling all necessary database
        /// interactions and error management.
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
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
        ///     repo.delete_by_id_with_executor(repo.pool(), user_id).await
        /// }
        /// ```
        #[inline(always)]
        async fn delete_by_id_with_executor<E>(
            &self,
            tx: E,
            id: impl Into<M::Id>
        ) -> crate::Result<()>
        where
            E: for<'c> Executor<'c, Database = Database>,
        {
             Self::delete_by_id_query(&id.into())
                .execute(tx)
                .await?;
            Ok(())
        }
    }

    /// Removes a single record from the database by its identifier.
    ///
    /// This method executes the deletion query generated by [`delete_by_id_query`](Self::delete_by_id_query). It provides
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
    #[inline(always)]
    async fn delete_by_id(&self, id: impl Into<M::Id>) -> crate::Result<()> {
        self.delete_by_id_with_executor(self.pool(), id).await
    }

    /// Deletes multiple records using the default batch size.
    ///
    /// This is a convenience wrapper around [`delete_batch`](Self::delete_batch_by_id) that uses [`DEFAULT_BATCH_SIZE`].
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
    #[inline(always)]
    async fn delete_many_by_id(&self, ids: impl IntoIterator<Item = M::Id>) -> crate::Result<()> {
        <Self as DeleteRepository<M>>::delete_batch_by_id::<DEFAULT_BATCH_SIZE>(self, ids).await
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "delete_batch_by_id",]
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
        /// 2. Processes each batch in a transaction using [`delete_query_by_id`](Self::delete_by_id_query)
        /// 3. Maintains ACID properties within each batch
        ///
        /// # Performance Considerations
        ///
        /// Consider batch size carefully:
        /// - Too small: More overhead from multiple transactions
        /// - Too large: Higher memory usage and longer transaction times
        #[inline(always)]
        async fn delete_batch_by_id<const N: usize>(
            &self,
            ids: impl IntoIterator<Item = M::Id>,
        ) -> crate::Result<()> {
            let span = tracing::Span::current();
            span.record("BATCH_SIZE", N);

            BatchOperator::<M::Id, N>::execute_query(ids, self.pool(), Self::delete_by_id_query).await
        }
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "delete_by_filter",]
        /// Removes records from the database by a filter.
        ///
        /// This method executes the deletion query generated by [`delete_by_filter_query`](Self::delete_by_filter_query)
        /// and uses the [`Executor`] `tx` for doing it. It provides a simple interface for removing records with a filter
        /// while handling all necessary database interactions and error management.
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `filter` - A [`SqlFilter`] to define what records should be deleted
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if the deletion was executed, or an error if the operation failed
        ///
        /// # Example
        ///
        /// ```no_compile
        /// async fn remove_user(repo: &UserRepository, filter: impl SqlFilter<'_>) -> crate::Result<()> {
        ///     repo.delete_by_filter_with_executor(repo.pool(), filter).await
        /// }
        /// ```
        #[inline(always)]
        async fn delete_by_filter_with_executor<E>(
            &self,
            tx: E,
            filter: impl SqlFilter<'_>
        ) -> crate::Result<()>
        where
            E: for<'c> Executor<'c, Database = Database>,
        {
            if !filter.should_apply_filter() {
                return Err(crate::Error::Repository {
                    message: "Can not Delete from table with a empty filter.".into(),
                })
            }

             Self::delete_by_filter_query(filter)
                .build()
                .execute(tx)
                .await?;
            Ok(())
        }
    }

    /// Removes records from the database by a filter.
    ///
    /// This method executes the deletion query generated by [`delete_by_filter_query`](Self::delete_by_filter_query)
    /// and uses the [`Repository::pool`] as a [`Executor`]. It provides a simple interface for removing records with a filter
    /// while handling all necessary database interactions and error management.
    ///
    /// # Parameters
    ///
    /// * `filter` - A [`SqlFilter`] to define what records should be deleted
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if the deletion was executed, or an error if the operation failed
    ///
    /// # Example
    ///
    /// ```no_compile
    /// async fn remove_user(repo: &UserRepository, filter: impl SqlFilter<'_>) -> crate::Result<()> {
    ///     repo.delete_by_filter(filter).await
    /// }
    /// ```
    #[inline(always)]
    async fn delete_by_filter(&self, filter: impl SqlFilter<'_>) -> crate::Result<()> {
        self.delete_by_filter_with_executor(self.pool(), filter)
            .await
    }
}
