//! Trait for automatically insert or update based on the presence of an ID.

use crate::prelude::Database;
use crate::traits::{InsertableRepository, Model, UpdatableRepository};
use crate::utils::{tracing_debug_log, BatchOperator, DEFAULT_BATCH_SIZE};
use sqlx::Executor;
use std::future::Future;
use std::pin::Pin;

/// Trait for repositories that can intelligently save records by either inserting or updating them.
///
/// The `SaveRepository` trait combines [`InsertableRepository`] and [`UpdatableRepository`]
/// to provide a higher-level interface for persisting models. It automatically determines
/// whether to insert or update a record based on whether the model has an ID, allowing
/// for simpler code when the operation type doesn't matter to the caller.
///
/// # Type Parameters
///
/// * `M` - The model type that this repository saves. Must implement the [`Model`] trait.
///
/// # Examples
///
/// The trait is automatically implemented for any repository that implements both
/// [`InsertableRepository`] and [`UpdatableRepository`]:
///
/// ```rust
/// # use sqlx_utils::traits::{Model, Repository, InsertableRepository, UpdatableRepository, SaveRepository};
/// # use sqlx_utils::types::{Pool, Query};
/// # struct User { id: Option<i32>, name: String }
/// # impl Model for User {
/// #     type Id = i32;
/// #     fn get_id(&self) -> Option<Self::Id> { self.id }
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
/// impl UpdatableRepository<User> for UserRepository {
///     fn update_query(user: &User) -> Query<'_> {
///         sqlx::query("UPDATE users SET name = $1 WHERE id = $2")
///             .bind(&user.name)
///             .bind(user.id.unwrap())
///     }
/// }
///
/// // SaveRepository is automatically implemented
///
/// // Usage
/// # async fn example(repo: &UserRepository) -> sqlx_utils::Result<()> {
/// // Create a new user (no ID)
/// let new_user = User { id: None, name: String::from("Alice") };
/// repo.save(new_user).await?; // Will insert
///
/// // Update an existing user (has ID)
/// let existing_user = User { id: Some(1), name: String::from("Updated Alice") };
/// repo.save_ref(&existing_user).await?; // Will update
///
/// // Save a mixed batch of new and existing users
/// let users = vec![
///     User { id: None, name: String::from("Bob") },        // Will insert
///     User { id: Some(2), name: String::from("Charlie") }  // Will update
/// ];
/// repo.save_all(users).await?; // Automatically sorts and batches operations
/// # Ok(())
/// # }
/// ```
///
/// # Implementation Notes
///
/// 1. This trait is automatically implemented for any type that implements both
///    [`InsertableRepository`] and [`UpdatableRepository`]
/// 2. The [`save`](SaveRepository::save_with_executor) method checks [`Model::get_id()`] to determine whether to insert or update
/// 3. The batch methods intelligently sort models into separate insert and update operations
/// 4. Where possible, insert and update operations within a batch are executed concurrently
///    for optimal performance
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `SaveRepository<{M}>` trait",
    label = "this type cannot automatically save `{M}` records",
    message = "`{Self}` must implement both `InsertableRepository<{M}>` and `UpdatableRepository<{M}>` to gain `SaveRepository<{M}>` capabilities"
)]
#[async_trait::async_trait]
pub trait SaveRepository<M: Model>: InsertableRepository<M> + UpdatableRepository<M> {
    tracing_debug_log! {
        [skip_all, Self::repository_span(), "save",]
        /// Intelligently persists a model instance by either inserting or updating using the [`Executor`] `tx`.
        ///
        /// This method determines the appropriate operation based on whether the model
        /// has an ID:
        /// - If the model has no ID, it performs an insertion
        /// - If the model has an ID, it performs an update
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `model` - A reference to the model instance to save
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if the operation was executed, or an error if it failed
        ///
        /// # Example
        ///
        /// ```no_compile
        /// async fn save_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
        ///     repo.save(user).await // Will insert or update based on user.id
        /// }
        /// ```
        #[inline]
        fn save_with_executor<'a, 'b, 'async_trait, E>(
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
                if model.get_id().is_none() {
                    <Self as InsertableRepository<M>>::insert_with_executor(self, tx, model).await
                } else {
                    <Self as UpdatableRepository<M>>::update_with_executor(self, tx, model).await
                }
            })
        }
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "save",]
        /// Intelligently persists a model instance by either inserting or updating using the [`Executor`] `tx`.
        ///
        /// This method determines the appropriate operation based on whether the model
        /// has an ID:
        /// - If the model has no ID, it performs an insertion
        /// - If the model has an ID, it performs an update
        ///
        /// # Parameters
        ///
        /// * `tx` - The executor to use for the query
        /// * `model` - A reference to the model instance to save
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if the operation was executed, or an error if it failed
        ///
        /// # Example
        ///
        /// ```no_compile
        /// async fn save_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
        ///     repo.save(user).await // Will insert or update based on user.id
        /// }
        /// ```
        #[inline]
        fn save_ref_with_executor<'a, 'b, 'c, 'async_trait, E>(
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
                if model.get_id().is_none() {
                    <Self as InsertableRepository<M>>::insert_ref_with_executor(self, tx, model).await
                } else {
                    <Self as UpdatableRepository<M>>::update_ref_with_executor(self, tx, model).await
                }
            })
        }
    }

    /// Intelligently persists a model instance by either inserting or updating.
    ///
    /// This method determines the appropriate operation based on whether the model
    /// has an ID:
    /// - If the model has no ID, it performs an insertion
    /// - If the model has an ID, it performs an update
    ///
    /// # Parameters
    ///
    /// * `model` - A reference to the model instance to save
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if the operation was executed, or an error if it failed
    ///
    /// # Example
    ///
    /// ```no_compile
    /// async fn save_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
    ///     repo.save(user).await // Will insert or update based on user.id
    /// }
    /// ```
    #[inline(always)]
    async fn save(&self, model: M) -> crate::Result<M>
    where
        M: 'async_trait,
    {
        self.save_with_executor(self.pool(), model).await
    }

    /// Intelligently persists a model instance by either inserting or updating.
    ///
    /// This method determines the appropriate operation based on whether the model
    /// has an ID:
    /// - If the model has no ID, it performs an insertion
    /// - If the model has an ID, it performs an update
    ///
    /// # Parameters
    ///
    /// * `model` - A reference to the model instance to save
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if the operation was executed, or an error if it failed
    ///
    /// # Example
    ///
    /// ```no_compile
    /// async fn save_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
    ///     repo.save(user).await // Will insert or update based on user.id
    /// }
    /// ```
    #[inline(always)]
    async fn save_ref(&self, model: &M) -> crate::Result<()>
    where
        M: 'async_trait,
    {
        self.save_ref_with_executor(self.pool(), model).await
    }

    /// Saves multiple models using the default batch size.
    ///
    /// This is a convenience wrapper around [`save_batch`](Self::save_batch) that uses [`DEFAULT_BATCH_SIZE`].
    /// It provides a simpler interface for bulk save operations when the default batch
    /// size is appropriate.
    ///
    /// # Parameters
    ///
    /// * `models` - An iterator yielding model instances to save
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if all operations were executed, or an error if any failed
    #[inline]
    async fn save_all<I>(&self, models: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = M> + Send + 'async_trait,
        I::IntoIter: Send,
    {
        <Self as SaveRepository<M>>::save_batch::<DEFAULT_BATCH_SIZE, I>(self, models).await
    }

    tracing_debug_log! {
        [skip_all, Self::repository_span(), "save_batch",]
        /// Performs an intelligent batched save operation with a specified batch size.
        ///
        /// This is the most sophisticated batch operation, efficiently handling both
        /// insertions and updates in the same operation. It sorts models based on
        /// whether they need insertion or update, then processes them optimally.
        ///
        /// # Type Parameters
        ///
        /// * `N` - The size of each batch to process
        ///
        /// # Parameters
        ///
        /// * `models` - An iterator yielding model instances to save
        ///
        /// # Returns
        ///
        /// * [`crate::Result<()>`](crate::Result) - Success if all batches were processed, or an error if any operation failed
        ///
        /// # Implementation Details
        ///
        /// The method:
        /// 1. Splits each batch into models requiring insertion vs update
        /// 2. Processes insertions and updates concurrently when possible
        /// 3. Handles empty cases efficiently
        /// 4. Maintains transactional integrity within each batch
        ///
        /// # Performance Features
        ///
        /// - Concurrent processing of inserts and updates
        /// - Efficient batch size management
        /// - Smart handling of empty cases
        /// - Transaction management for data consistency
        ///
        /// # Performance Considerations
        ///
        /// Consider batch size carefully:
        /// - Too small: More overhead from multiple transactions
        /// - Too large: Higher memory usage and longer transactions times
        fn save_batch<'a, 'async_trait, const N: usize, I>(
            &'a self,
            models: I,
        ) -> Pin<Box<dyn Future<Output = crate::Result<()>> + Send + 'async_trait>>
        where
            I: IntoIterator<Item = M> + Send + 'async_trait,
            I::IntoIter: Send,
            'a: 'async_trait,
            Self: 'async_trait,
            M: 'async_trait,
        {
            let span = tracing::Span::current();
            span.record("BATCH_SIZE", N);

            let op = BatchOperator::<M, N>::execute_batch(models, |batch| async {
                let mut update = Vec::new();
                let mut insert = Vec::new();

                for model in batch {
                    if model.get_id().is_some() {
                        update.push(model);
                    } else {
                        insert.push(model);
                    }
                }

                match (update.is_empty(), insert.is_empty()) {
                    (false, false) => {
                        futures::try_join!(<Self as UpdatableRepository<M>>::update_batch::<N, Vec<M>>(self, update), <Self as InsertableRepository<M>>::insert_batch::<N, Vec<M>>(self, insert))?;
                    }
                    (false, true) => {
                        <Self as UpdatableRepository<M>>::update_batch::<N, Vec<M>>(self, update).await?;
                    }
                    (true, false) => {
                        <Self as InsertableRepository<M>>::insert_batch::<N, Vec<M>>(self, insert).await?;
                    }
                    (true, true) => {}
                }

                Ok(())
            });

            Box::pin(async move {
                op.await
            })
        }
    }
}

#[async_trait::async_trait]
impl<M: Model, T: InsertableRepository<M> + UpdatableRepository<M>> SaveRepository<M> for T {}
