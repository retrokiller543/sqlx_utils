//! Trait for adding insert capabilities to a repository

use crate::prelude::Database;
use crate::traits::{Model, Repository};
use crate::types::Query;
use crate::utils::{tracing_debug_log, BatchOperator, DEFAULT_BATCH_SIZE};
use sqlx::Executor;

/// Trait for repositories that can insert new records into the database.
///
/// The `InsertableRepository` trait extends the base [`Repository`] trait with methods
/// for inserting new records. It provides standardized ways to insert both individual models
/// and batches of models, optimizing database interactions for performance while maintaining
/// data integrity.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository inserts. Must implement the [`Model`] trait.
///
/// # Examples
///
/// Basic implementation:
/// ```rust
/// # use sqlx_utils::traits::{Model, Repository, InsertableRepository};
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
/// impl InsertableRepository<User> for UserRepository {
///     fn insert_query(user: &User) -> Query<'_> {
///         sqlx::query("INSERT INTO users (name) VALUES ($1)")
///             .bind(&user.name)
///     }
/// }
///
/// // Usage
/// # async fn example(repo: &UserRepository, user: &User) -> sqlx_utils::Result<()> {
/// // Insert a single user
/// repo.insert(user).await?;
///
/// // Insert multiple users
/// let users = vec![
///     User { id: 1, name: String::from("Alice") },
///     User { id: 2, name: String::from("Bob") }
/// ];
/// repo.insert_many(users).await?;
/// # Ok(())
/// # }
/// ```
///
/// Using the macro for simpler implementation:
/// ```rust
/// # use sqlx_utils::{repository, repository_insert};
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
/// repository_insert! {
///     UserRepository<User>;
///
///     insert_query(user) {
///         sqlx::query("INSERT INTO users (name) VALUES ($1)")
///             .bind(&user.name)
///     }
/// }
/// ```
///
/// # Implementation Notes
///
/// 1. Required method: [`insert_query`](InsertableRepository::insert_query) - Defines how a model is translated into an INSERT statement
/// 2. Provided methods:
///    - [`insert_with_executor`](InsertableRepository::insert_with_executor) - Inserts a single model using any [`Executor`]
///    - [`insert`](InsertableRepository::insert) - Inserts a single model
///    - [`insert_many`](InsertableRepository::insert_many) - Inserts multiple models using the default batch size
///    - [`insert_batch`](InsertableRepository::insert_batch) - Inserts multiple models with a custom batch size
/// 3. All batch operations use transactions to ensure data consistency
/// 4. Performance is optimized through batching and connection pooling
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `InsertableRepository<{M}>` trait",
    label = "this type does not implement `InsertableRepository` for model type `{M}`",
    message = "`{Self}` must implement `InsertableRepository<{M}>` to insert `{M}` records"
)]
pub trait InsertableRepository<M: Model>: Repository<M> {
    /// Creates a SQL query to insert a single model instance into the database.
    ///
    /// This method defines how a model should be persisted in the database as a new record.
    /// It constructs a parameterized query that maps the model's fields to database columns.
    /// The query is returned without being executed, allowing for transaction management
    /// and error handling at a higher level.
    ///
    /// # Parameters
    ///
    /// * `model` - A reference to the model instance to be inserted
    ///
    /// # Returns
    ///
    /// * [`Query`] - A prepared SQL query ready for execution
    ///
    /// # Implementation Notes
    ///
    /// The implementing repository should:
    /// 1. Handle all model fields appropriately
    /// 2. Use proper SQL parameter binding for safety
    /// 3. Return an appropriate error if the model is invalid
    fn insert_query(model: &M) -> Query<'_>;

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "insert",]
        /// Persists a new model instance to the database.
        ///
        /// This method executes the insertion query generated by [`insert_query`](InsertableRepository::insert_query) with the [`Executor`] `tx`. It handles
        /// the actual database interaction and provides a simple interface for creating new records.
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `model` - A reference to the model instance to insert
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if the insertion was executed, or an error if the operation failed
        ///
        /// # Example
        ///
        /// ```no_compile
        /// async fn create_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
        ///     repo.insert_with_executor(repo.pool(), user).await
        /// }
        /// ```
        ///
        /// # Panics
        ///
        /// The method will panic if an ID is present, but it will only do so in debug mode to avoid
        /// performance issues. This is so that we don't insert a duplicate key, if this is the desired behavior you want you can enable the feature `insert_duplicate`
        #[inline(always)]
        async fn insert_with_executor<E>(
            &self,
            tx: E,
            model: &M
        ) -> crate::Result<()>
        where
            E: for<'c> Executor<'c, Database = Database>,
        {
            #[cfg(not(feature = "insert_duplicate"))]
            debug_assert!(model.get_id().is_none());

             Self::insert_query(model)
                .execute(tx)
                .await?;
            Ok(())
        }
    }

    /// Persists a new model instance to the database.
    ///
    /// This method executes the insertion query generated by [`insert_query`](InsertableRepository::insert_query). It handles
    /// the actual database interaction and provides a simple interface for creating
    /// new records.
    ///
    /// # Parameters
    ///
    /// * `model` - A reference to the model instance to insert
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if the insertion was executed, or an error if the operation failed
    ///
    /// # Example
    ///
    /// ```no_compile
    /// async fn create_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
    ///     repo.insert(user).await
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// The method will panic if an ID is present, but it will only do so in debug mode to avoid
    /// performance issues. This is so that we don't insert a duplicate key, if this is the desired behavior you want you can enable the feature `insert_duplicate`
    #[inline(always)]
    async fn insert(&self, model: &M) -> crate::Result<()> {
        self.insert_with_executor(self.pool(), model).await
    }

    /// Inserts multiple models using the default batch size.
    ///
    /// This is a convenience wrapper around [`insert_batch`](InsertableRepository::insert_batch) that uses [`DEFAULT_BATCH_SIZE`].
    /// It provides a simpler interface for bulk insertions when the default batch size
    /// is appropriate for the use case.
    ///
    /// # Parameters
    ///
    /// * `models` - An iterator yielding model instances to insert
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if all insertions were executed, or an error if any operation failed
    ///
    /// # Example
    ///
    /// ```no_compile
    /// async fn create_users(repo: &UserRepository, users: Vec<User>) -> crate::Result<()> {
    ///     repo.insert_many(users).await
    /// }
    /// ```
    #[inline(always)]
    async fn insert_many(&self, models: impl IntoIterator<Item = M>) -> crate::Result<()> {
        <Self as InsertableRepository<M>>::insert_batch::<DEFAULT_BATCH_SIZE>(self, models).await
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "delete_batch_by_id",]
        /// Performs a batched insertion operation with a specified batch size.
        ///
        /// This method uses [`BatchOperator`] to efficiently process large numbers of insertions
        /// in chunks. It helps prevent memory overflow and maintains optimal database performance
        /// by limiting the number of records processed at once.
        ///
        /// # Type Parameters
        ///
        /// * `N` - The size of each batch to process
        ///
        /// # Parameters
        ///
        /// * `models` - An iterator yielding model instances to insert
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if all batches were processed, or an error if any operation failed
        ///
        /// # Implementation Details
        ///
        /// The method:
        /// 1. Chunks the input into batches of size N
        /// 2. Processes each batch in a transaction
        /// 3. Uses the [`insert_query`](InsertableRepository::insert_query) query for each model
        /// 4. Maintains ACID properties within each batch
        ///
        /// # Performance Considerations
        ///
        /// Consider batch size carefully:
        /// - Too small: More overhead from multiple transactions
        /// - Too large: Higher memory usage and longer transaction times
        #[inline(always)]
        async fn insert_batch<const N: usize>(
            &self,
            models: impl IntoIterator<Item = M>,
        ) -> crate::Result<()> {
            let span = tracing::Span::current();
            span.record("BATCH_SIZE", N);

            BatchOperator::<M, N>::execute_query(models, self.pool(), Self::insert_query).await
        }

    }
}
