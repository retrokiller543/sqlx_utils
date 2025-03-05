use crate::traits::{Model, Repository};
use crate::types::Query;
use crate::utils::{BatchOperator, DEFAULT_BATCH_SIZE};

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
    /// * [`Query`] - A prepared PostgreSQL query ready for execution
    ///
    /// # Implementation Notes
    ///
    /// The implementing repository should:
    /// 1. Handle all model fields appropriately
    /// 2. Use proper SQL parameter binding for safety
    /// 3. Return an appropriate error if the model is invalid
    fn insert_query(model: &M) -> Query<'_>;

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
    #[tracing::instrument(skip_all, level = "debug")]
    async fn insert(&self, model: &M) -> crate::Result<()> {
        Self::insert_query(model).execute(self.pool()).await?;
        Ok(())
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
    #[tracing::instrument(skip_all, level = "debug")]
    async fn insert_many(&self, models: impl IntoIterator<Item = M>) -> crate::Result<()> {
        <Self as InsertableRepository<M>>::insert_batch::<DEFAULT_BATCH_SIZE>(self, models).await
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
    /// 3. Uses the [`insert_query`](InsertableRepository::insert_query) query for each model
    /// 4. Maintains ACID properties within each batch
    ///
    /// # Performance Considerations
    ///
    /// Consider batch size carefully:
    /// - Too small: More overhead from multiple transactions
    /// - Too large: Higher memory usage and longer transaction times
    #[tracing::instrument(skip_all, level = "debug")]
    async fn insert_batch<const N: usize>(
        &self,
        models: impl IntoIterator<Item = M>,
    ) -> crate::Result<()> {
        BatchOperator::<M, N>::execute_query(models, self.pool(), Self::insert_one).await
    }
}