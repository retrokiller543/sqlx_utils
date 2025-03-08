//! Trait for adding update capabilities to a repository

use crate::prelude::Database;
use crate::traits::{Model, Repository};
use crate::types::Query;
use crate::utils::{tracing_debug_log, BatchOperator, DEFAULT_BATCH_SIZE};
use sqlx::Executor;
use std::future::Future;
use std::pin::Pin;

/// Trait for repositories that can update existing records in the database.
///
/// The `UpdatableRepository` trait extends the base [`Repository`] trait with methods
/// for updating existing records. It provides standardized ways to update both individual
/// models and batches of models, optimizing database interactions while maintaining data
/// integrity.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository updates. Must implement the [`Model`] trait.
///
/// # Examples
///
/// Basic implementation:
/// ```rust
/// # use sqlx_utils::traits::{Model, Repository, UpdatableRepository};
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
/// impl UpdatableRepository<User> for UserRepository {
///     fn update_query(user: &User) -> Query<'_> {
///         sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
///             .bind(&user.name)
///             .bind(user.id)
///     }
/// }
///
/// // Usage
/// # async fn example(repo: &UserRepository, user: &User) -> sqlx_utils::Result<()> {
/// // Update a single user
/// repo.update(user).await?;
///
/// // Update multiple users
/// let users = vec![
///     User { id: 1, name: String::from("Updated Alice") },
///     User { id: 2, name: String::from("Updated Bob") }
/// ];
/// repo.update_many(users).await?;
/// # Ok(())
/// # }
/// ```
///
/// Using the macro for simpler implementation:
/// ```rust
/// # use sqlx_utils::{repository, repository_update};
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
/// repository_update! {
///     UserRepository<User>;
///
///     update_query(user) {
///         sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
///             .bind(&user.name)
///             .bind(user.id)
///     }
/// }
/// ```
///
/// # Implementation Notes
///
/// 1. Required method: [`update_query`](UpdatableRepository::update_query) - Defines how a model is translated into an UPDATE statement
/// 2. Provided methods:
///    - [`update_with_executor`](UpdatableRepository::update_with_executor) - updates a single model using any [`Executor`]
///    - [`update`](UpdatableRepository::update) - Updates a single model
///    - [`update_many`](UpdatableRepository::update_many) - Updates multiple models using the default batch size
///    - [`update_batch`](UpdatableRepository::update_batch) - Updates multiple models with a custom batch size
/// 3. All batch operations use transactions to ensure data consistency
/// 4. All methods assume the model has a valid ID (typically checked by the `Model::get_id` method)
/// 5. Performance is optimized through batching and connection pooling
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `UpdatableRepository<{M}>` trait",
    label = "this type does not implement `UpdatableRepository` for model type `{M}`",
    message = "`{Self}` must implement `UpdatableRepository<{M}>` to update `{M}` records"
)]
#[async_trait::async_trait]
pub trait UpdatableRepository<M: Model>: Repository<M> {
    /// Creates a SQL query to update an existing model in the database.
    ///
    /// This method constructs an UPDATE statement that will modify an existing database record
    /// to match the current state of the model. It should use the model's ID to identify
    /// the record to update and include all relevant fields in the SET clause.
    ///
    /// # Parameters
    ///
    /// * `model` - A reference to the model instance containing updated values
    ///
    /// # Returns
    ///
    /// * [`Query`] - A prepared SQL UPDATE query
    ///
    /// # Implementation Notes
    ///
    /// The query should:
    /// 1. Include a WHERE clause matching the model's ID
    /// 2. Only update columns that can be modified
    /// 3. Preserve any timestamp or audit fields as required
    fn update_query(model: &M) -> Query<'_>;

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "update",]
        /// Executes an update operation for a single model instance.
        ///
        /// This method takes the query generated by [`update_query`](Self::update_query) and executes it against the [`Executor`] `tx`.
        /// It's a higher-level wrapper that handles the actual database interaction, providing a simpler
        /// interface for updating records.
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `model` - A reference to the model instance to update
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if the update was executed, or an error if the operation failed
        ///
        /// # Implementation Details
        ///
        /// The method:
        /// 1. Gets the update query from [`update_query`](Self::update_query)
        /// 2. Executes it using the connection pool
        /// 3. Handles any potential database errors
        #[inline(always)]
        fn update_with_executor<'a, 'b, 'async_trait, E>(
            &'a self,
            tx: E,
            model: M
        ) -> Pin<
            Box<
                dyn Future<
                    Output = crate::Result<M>,
                > + Send + 'async_trait,
            >,
        >
        where
            'a: 'async_trait,
            Self: 'async_trait,
            M: 'async_trait,
            E: Executor<'b, Database = Database>+ 'async_trait,
        {
            Box::pin(async move {
                Self::update_query(&model).execute(tx).await?;
                Ok(model)
            })
        }
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "update",]
        /// Executes an update operation for a single model instance.
        ///
        /// This method takes the query generated by [`update_query`](Self::update_query) and executes it against the [`Executor`] `tx`.
        /// It's a higher-level wrapper that handles the actual database interaction, providing a simpler
        /// interface for updating records.
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `model` - A reference to the model instance to update
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if the update was executed, or an error if the operation failed
        ///
        /// # Implementation Details
        ///
        /// The method:
        /// 1. Gets the update query from [`update_query`](Self::update_query)
        /// 2. Executes it using the connection pool
        /// 3. Handles any potential database errors
        #[inline(always)]
        fn update_ref_with_executor<'a, 'b, 'c, 'async_trait, E>(
            &'a self,
            tx: E,
            model: &'b M
        ) -> Pin<
            Box<
                dyn Future<
                    Output = crate::Result<()>,
                > + Send + 'async_trait,
            >,
        >
        where
            'a: 'async_trait,
            'b: 'async_trait,
            Self: 'async_trait,
            M: 'async_trait,
            E: Executor<'c, Database = Database>+ 'async_trait,
        {
            Box::pin(async move {
                Self::update_query(model).execute(tx).await?;
                Ok(())
            })
        }
    }

    /// Executes an update operation for a single model instance.
    ///
    /// This method takes the query generated by [`update_query`](Self::update_query) and executes it against the database.
    /// It's a higher-level wrapper that handles the actual database interaction, providing a simpler
    /// interface for updating records.
    ///
    /// # Parameters
    ///
    /// * `model` - A reference to the model instance to update
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if the update was executed, or an error if the operation failed
    ///
    /// # Implementation Details
    ///
    /// The method:
    /// 1. Gets the update query from [`update_query`](Self::update_query)
    /// 2. Executes it using the connection pool
    /// 3. Handles any potential database errors
    #[inline(always)]
    async fn update(&self, model: M) -> crate::Result<M>
    where
        M: 'async_trait,
    {
        self.update_with_executor(self.pool(), model).await
    }

    /// Executes an update operation for a single model instance.
    ///
    /// This method takes the query generated by [`update_query`](Self::update_query) and executes it against the database.
    /// It's a higher-level wrapper that handles the actual database interaction, providing a simpler
    /// interface for updating records.
    ///
    /// # Parameters
    ///
    /// * `model` - A reference to the model instance to update
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if the update was executed, or an error if the operation failed
    ///
    /// # Implementation Details
    ///
    /// The method:
    /// 1. Gets the update query from [`update_query`](Self::update_query)
    /// 2. Executes it using the connection pool
    /// 3. Handles any potential database errors
    #[inline(always)]
    async fn update_ref(&self, model: &M) -> crate::Result<()>
    where
        M: 'async_trait,
    {
        self.update_ref_with_executor(self.pool(), model).await
    }

    /// Updates multiple models using the default batch size.
    ///
    /// This is a convenience wrapper around [`update_batch`](Self::update_batch) that uses [`DEFAULT_BATCH_SIZE`].
    /// It simplifies bulk update operations when the default batch size is suitable.
    ///
    /// # Parameters
    ///
    /// * `models` - An iterator yielding model instances to update
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if all updates were executed, or an error if any operation failed
    #[inline(always)]
    async fn update_many<I>(&self, models: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = M> + Send + 'async_trait,
        I::IntoIter: Send,
    {
        self.update_batch::<DEFAULT_BATCH_SIZE, I>(models).await
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "update_batch",]
        /// Performs a batched update operation with a specified batch size.
        ///
        /// Similar to [`insert_batch`](crate::traits::InsertableRepository::insert_batch), this method uses [`BatchOperator`] to efficiently process
        /// large numbers of updates in chunks, preventing memory overflow and maintaining
        /// optimal database performance.
        ///
        /// # Type Parameters
        ///
        /// * `N` - The size of each batch to process
        ///
        /// # Parameters
        ///
        /// * `models` - An iterator yielding model instances to update
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if all batches were processed, or an error if any operation failed
        ///
        /// # Performance Considerations
        ///
        /// Consider batch size carefully:
        /// - Too small: More overhead from multiple transactions
        /// - Too large: Higher memory usage and longer transactions times
        #[inline(always)]
        fn update_batch<'a, 'async_trait, const N: usize, I>(
            &'a self,
            models: I,
        ) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + 'async_trait>>
        where
            I: IntoIterator<Item = M> + Send + 'async_trait,
            I::IntoIter: Send,
            'a: 'async_trait,
            Self: 'async_trait,
        {
            let span = tracing::Span::current();
            span.record("BATCH_SIZE", N);

            Box::pin(async move {
                BatchOperator::<M, N>::execute_query(models, self.pool(), Self::update_query).await
            })
        }
    }
}
