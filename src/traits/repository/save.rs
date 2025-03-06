//! Trait for automatically insert or update based on the presence of an ID.

use crate::traits::{InsertableRepository, Model, UpdatableRepository};
use crate::utils::{BatchOperator, DEFAULT_BATCH_SIZE};

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
/// repo.save(&new_user).await?; // Will insert
///
/// // Update an existing user (has ID)
/// let existing_user = User { id: Some(1), name: String::from("Updated Alice") };
/// repo.save(&existing_user).await?; // Will update
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
/// 2. The [`save`](SaveRepository::save) method checks [`Model::get_id()`] to determine whether to insert or update
/// 3. The batch methods intelligently sort models into separate insert and update operations
/// 4. Where possible, insert and update operations within a batch are executed concurrently
///    for optimal performance
#[diagnostic::on_unimplemented(
    note = "Type `{Self}` does not implement the `SaveRepository<{M}>` trait",
    label = "this type cannot automatically save `{M}` records",
    message = "`{Self}` must implement both `InsertableRepository<{M}>` and `UpdatableRepository<{M}>` to gain `SaveRepository<{M}>` capabilities"
)]
pub trait SaveRepository<M: Model>: InsertableRepository<M> + UpdatableRepository<M> {
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
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn save(&self, model: &M) -> crate::Result<()> {
        if model.get_id().is_none() {
            <Self as InsertableRepository<M>>::insert(self, model).await
        } else {
            <Self as UpdatableRepository<M>>::update(self, model).await
        }
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
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    #[inline]
    async fn save_all(&self, models: impl IntoIterator<Item = M>) -> crate::Result<()> {
        <Self as SaveRepository<M>>::save_batch::<DEFAULT_BATCH_SIZE>(self, models).await
    }

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
    /// - Too large: Higher memory usage and longer transaction times
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn save_batch<const N: usize>(
        &self,
        models: impl IntoIterator<Item = M>,
    ) -> crate::Result<()> {
        BatchOperator::<M, N>::execute_batch(models, |batch| async {
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
                    futures::try_join!(<Self as UpdatableRepository<M>>::update_batch::<N>(self, update), <Self as InsertableRepository<M>>::insert_batch::<N>(self, insert))?;
                }
                (false, true) => {
                    <Self as UpdatableRepository<M>>::update_batch::<N>(self, update).await?;
                }
                (true, false) => {
                    <Self as InsertableRepository<M>>::insert_batch::<N>(self, insert).await?;
                }
                (true, true) => {}
            }

            Ok(())
        })
            .await
    }
}

impl<M: Model, T: InsertableRepository<M> + UpdatableRepository<M>> SaveRepository<M> for T {}
