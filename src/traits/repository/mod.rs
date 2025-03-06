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
    /// * `&`[`PgPool`] - A reference to the PostgreSQL connection pool
    ///
    /// # Example
    ///
    /// ```rust
    /// let repo = UserRepository::new(pool.clone());
    /// let pool_ref = repo.pool();
    /// // Use pool_ref for custom database operations
    /// ```
    fn pool(&self) -> &crate::types::Pool;

    #[inline]
    fn repository_span() -> Span {
        debug_span!("Repository")
    }

    /*/// Creates a SQL query to insert a single model instance into the database.
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
    /// * [`Query`] - A prepared PostgreSQL query ready for execution
    ///
    /// # Implementation Notes
    ///
    /// The implementing repository should:
    /// 1. Handle all model fields appropriately
    /// 2. Use proper SQL parameter binding for safety
    /// 3. Return an appropriate error if the model is invalid
    fn insert_one(model: &M) -> Query<'_>;

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
    /// * [Query] - A prepared PostgreSQL UPDATE query
    ///
    /// # Implementation Notes
    ///
    /// The query should:
    /// 1. Include a WHERE clause matching the model's ID
    /// 2. Only update columns that can be modified
    /// 3. Preserve any timestamp or audit fields as required
    fn update_one(model: &M) -> Query<'_>;

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
    fn delete_one_by_id(id: &M::Id) -> Query<'_>;*/


    /*/// Retrieves all records of this model type from the database.
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
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn get_all(&self) -> crate::Result<Vec<M>> {
        unimplemented!("This method has not been implemented for this repository")
    }

    /// Gets by a filter
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn get_by_any_filter(&self, _filter: impl SqlFilter<'_>) -> crate::Result<Vec<M>> {
        unimplemented!("This method has not been implemented for this repository")
    }

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
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn get_by_id(&self, _: impl Into<M::Id>) -> crate::Result<Option<M>> {
        unimplemented!("This method has not been implemented for this repository")
    }

    /// Persists a new model instance to the database.
    ///
    /// This method executes the insertion query generated by [`insert_one`](Repository::insert_one). It handles
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
    /// ```rust
    /// async fn create_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
    ///     repo.insert(user).await
    /// }
    /// ```
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn insert(&self, model: &M) -> crate::Result<()> {
        Self::insert_one(model).execute(self.pool()).await?;
        Ok(())
    }

    /// Inserts multiple models using the default batch size.
    ///
    /// This is a convenience wrapper around [`insert_batch`](Repository::insert_batch) that uses [`DEFAULT_BATCH_SIZE`].
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
    /// ```rust
    /// async fn create_users(repo: &UserRepository, users: Vec<User>) -> crate::Result<()> {
    ///     repo.insert_many(users).await
    /// }
    /// ```
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn insert_many(&self, models: impl IntoIterator<Item = M>) -> crate::Result<()> {
        self.insert_batch::<DEFAULT_BATCH_SIZE>(models).await
    }

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
    /// 3. Uses the [`insert_one`](Repository::insert_one) query for each model
    /// 4. Maintains ACID properties within each batch
    ///
    /// # Performance Considerations
    ///
    /// Consider batch size carefully:
    /// - Too small: More overhead from multiple transactions
    /// - Too large: Higher memory usage and longer transaction times
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn insert_batch<const N: usize>(
        &self,
        models: impl IntoIterator<Item = M>,
    ) -> crate::Result<()> {
        BatchOperator::<M, N>::execute_query(models, self.pool(), Self::insert_one).await
    }

    /// Executes an update operation for a single model instance.
    ///
    /// This method takes the query generated by [`update_one`](Repository::update_one) and executes it against the database.
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
    /// 1. Gets the update query from [`update_one`](Repository::update_one)
    /// 2. Executes it using the connection pool
    /// 3. Handles any potential database errors
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn update(&self, model: &M) -> crate::Result<()> {
        Self::update_one(model).execute(self.pool()).await?;
        Ok(())
    }

    /// Updates multiple models using the default batch size.
    ///
    /// This is a convenience wrapper around [`update_batch`](Repository::update_batch) that uses [`DEFAULT_BATCH_SIZE`].
    /// It simplifies bulk update operations when the default batch size is suitable.
    ///
    /// # Parameters
    ///
    /// * `models` - An iterator yielding model instances to update
    ///
    /// # Returns
    ///
    /// * [`crate::Result<()>`](crate::Result) - Success if all updates were executed, or an error if any operation failed
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn update_many(&self, models: impl IntoIterator<Item = M>) -> crate::Result<()> {
        self.update_batch::<DEFAULT_BATCH_SIZE>(models).await
    }

    /// Performs a batched update operation with a specified batch size.
    ///
    /// Similar to [`insert_batch`](Repository::insert_batch), this method uses [`BatchOperator`] to efficiently process
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
    /// - Too large: Higher memory usage and longer transaction times
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn update_batch<const N: usize>(
        &self,
        models: impl IntoIterator<Item = M>,
    ) -> crate::Result<()> {
        BatchOperator::<M, N>::execute_query(models, self.pool(), Self::update_one).await
    }

    /// Removes a single record from the database by its identifier.
    ///
    /// This method executes the deletion query generated by [`delete_one_by_id`](Repository::delete_one_by_id). It provides
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
    /// ```rust
    /// async fn remove_user(repo: &UserRepository, user_id: i32) -> crate::Result<()> {
    ///     repo.delete_by_id(user_id).await
    /// }
    /// ```
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn delete_by_id(&self, id: impl Into<M::Id>) -> crate::Result<()> {
        Self::delete_one_by_id(&id.into())
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Deletes multiple records using the default batch size.
    ///
    /// This is a convenience wrapper around [`delete_batch`](Repository::delete_batch) that uses [`DEFAULT_BATCH_SIZE`].
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
        self.delete_batch::<DEFAULT_BATCH_SIZE>(ids).await
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
    /// 2. Processes each batch in a transaction using [`delete_one_by_id`](Repository::delete_one_by_id)
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
        BatchOperator::<M::Id, N>::execute_query(ids, self.pool(), Self::delete_one_by_id).await
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
    /// ```rust
    /// async fn save_user(repo: &UserRepository, user: &User) -> crate::Result<()> {
    ///     repo.save(user).await // Will insert or update based on user.id
    /// }
    /// ```
    #[tracing::instrument(skip_all, level = "debug", parent = Self::repository_span())]
    async fn save(&self, model: &M) -> crate::Result<()> {
        if model.get_id().is_none() {
            self.insert(model).await
        } else {
            self.update(model).await
        }
    }

    /// Saves multiple models using the default batch size.
    ///
    /// This is a convenience wrapper around [`save_batch`](Repository::save_batch) that uses [`DEFAULT_BATCH_SIZE`].
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
    fn save_all(&self, models: impl IntoIterator<Item = M>) -> impl Future<Output = crate::Result<()>> {
        async { self.save_batch::<DEFAULT_BATCH_SIZE>(models).await }
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
                // Both non-empty => run them concurrently
                (false, false) => {
                    futures::try_join!(self.update_many(update), self.insert_many(insert))?;
                }
                // Only update
                (false, true) => {
                    self.update_many(update).await?;
                }
                // Only insert
                (true, false) => {
                    self.insert_many(insert).await?;
                }
                // Neither => no-op
                (true, true) => {}
            }

            Ok(())
        })
            .await
    }*/
}
