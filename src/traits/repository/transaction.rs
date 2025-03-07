use crate::prelude::SaveRepository;
use crate::{
    traits::{Model, Repository},
    types::Database,
};
use futures::future::try_join_all;
use sqlx::{Error, Transaction};
use std::future::Future;
use std::sync::Arc;

/// Extension trait for Repository to work with transactions
///
/// This trait adds transaction capabilities to any repository that implements
/// the [`Repository`] trait. It provides several methods for executing operations
/// within database transactions, with different strategies for concurrency and
/// error handling.
///
/// The trait is automatically implemented for any type that implements [`Repository<M>`],
/// making transaction capabilities available to all repositories without additional code.
pub trait TransactionRepository<M>: Repository<M>
where
    M: Model,
{
    /// Executes a callback within a transaction, handling the transaction lifecycle automatically.
    ///
    /// This method:
    /// 1. Begins a transaction from the repository's connection pool
    /// 2. Passes the transaction to the callback function
    /// 3. Waits for the callback to complete and return both a result and the transaction
    /// 4. Commits the transaction if the result is `Ok`, or rolls it back if it's `Err`
    /// 5. Returns the final result
    ///
    /// # Type Parameters
    ///
    /// * `F`: The type of the callback function
    /// * `Fut`: The future type returned by the callback
    /// * `R`: The result type
    /// * `E`: The error type, which must be convertible from [`Error`]
    ///
    /// # Parameters
    ///
    /// * `callback`: A function that accepts a [`Transaction`] and returns a future
    ///
    /// # Returns
    ///
    /// A future that resolves to `Result<R, E>`.
    ///
    /// # Example
    ///
    /// ```no_compile
    /// let result = repo.with_transaction(|mut tx| async move {
    ///     let model = Model::new();
    ///     let res = repo.save_with_executor(&mut tx, model).await;
    ///     (res, tx)
    /// }).await;
    /// ```
    fn with_transaction<'a, 'b, F, Fut, R, E>(
        &'a self,
        callback: F,
    ) -> impl Future<Output = Result<R, E>> + Send + 'a
    where
        F: FnOnce(Transaction<'b, Database>) -> Fut + Send + 'a,
        Fut: Future<Output = (Result<R, E>, Transaction<'b, Database>)> + Send,
        R: Send + 'a,
        E: From<Error> + Send,
    {
        async move {
            let transaction = self.pool().begin().await.map_err(E::from)?;

            let (ret, tx) = callback(transaction).await;

            match ret {
                Ok(val) => {
                    tx.commit().await.map_err(E::from)?;
                    Ok(val)
                }
                Err(err) => {
                    tx.rollback().await.map_err(E::from)?;
                    Err(err)
                }
            }
        }
    }

    /// Executes multiple operations sequentially in a transaction, stopping at the first error.
    ///
    /// This method provides an optimized approach for cases where you want to stop processing
    /// as soon as any action fails, immediately rolling back the transaction.
    ///
    /// # Type Parameters
    ///
    /// * `I`: The iterator type
    /// * `F`: The action function type
    /// * `Fut`: The future type returned by each action
    /// * `R`: The result type
    /// * `E`: The error type, which must be convertible from [`Error`]
    ///
    /// # Parameters
    ///
    /// * `actions`: An iterator of functions that will be executed in the transaction
    ///
    /// # Returns
    ///
    /// A future that resolves to:
    /// * `Ok(Vec<R>)`: A vector of results if all actions succeeded
    /// * `Err(E)`: The first error encountered
    ///
    /// # Implementation Details
    ///
    /// 1. Begins a transaction from the repository's connection pool
    /// 2. Executes each action sequentially, collecting results
    /// 3. If any action fails, rolls back the transaction and returns the error
    /// 4. If all actions succeed, commits the transaction and returns the results
    ///
    /// # Example
    ///
    /// ```no_compile
    /// let results = repo.transaction_sequential([
    ///     |tx| async move { repo.save_with_executor(tx, model1).await },
    ///     |tx| async move { repo.save_with_executor(tx, model2).await }
    /// ]).await;
    /// ```
    fn transaction_sequential<'a, 'b, I, F, Fut, R, E>(
        &'a self,
        actions: I,
    ) -> impl Future<Output = Result<Vec<R>, E>> + Send + 'a
    where
        I: IntoIterator<Item = F> + Send + 'a,
        I::IntoIter: Send + 'a,
        F: FnOnce(Transaction<'b, Database>) -> Fut + Send + 'a,
        Fut: Future<Output = (Result<R, E>, Transaction<'b, Database>)> + Send,
        R: Send + 'a,
        E: From<Error> + Send + 'a,
    {
        async move {
            let mut tx = self.pool().begin().await.map_err(E::from)?;
            let mut results = Vec::new();

            for action in actions {
                let (result, new_tx) = action(tx).await;
                tx = new_tx; // Get back ownership

                match result {
                    Ok(value) => results.push(value),
                    Err(e) => {
                        let _ = tx.rollback().await;
                        return Err(e);
                    }
                }
            }

            tx.commit().await.map_err(E::from)?;
            Ok(results)
        }
    }

    /// Executes multiple operations concurrently in a transaction.
    ///
    /// This method allows for concurrent execution of actions within a transaction,
    /// which can significantly improve performance for I/O-bound operations.
    /// Note that this only works when the actions don't have data dependencies.
    ///
    /// # Type Parameters
    ///
    /// * `I`: The iterator type
    /// * `F`: The action function type
    /// * `Fut`: The future type returned by each action
    /// * `R`: The result type
    /// * `E`: The error type, which must be convertible from [`Error`]
    ///
    /// # Parameters
    ///
    /// * `action_fns`: An iterator of functions that will be executed concurrently in the transaction
    ///
    /// # Returns
    ///
    /// A future that resolves to:
    /// * `Ok(Vec<R>)`: A vector of results if all actions succeeded
    /// * `Err(E)`: The first error encountered
    ///
    /// # Implementation Details
    ///
    /// 1. Begins a transaction from the repository's connection pool
    /// 2. Wraps the transaction in an [`Arc<Mutex<_>>`] to safely share it between concurrent operations
    /// 3. Creates futures for all actions but doesn't execute them yet
    /// 4. Executes all futures concurrently using [`try_join_all`]
    /// 5. If all operations succeed, commits the transaction and returns the results
    /// 6. If any operation fails, rolls back the transaction and returns the error
    ///
    /// # Notes
    ///
    /// - Uses [`parking_lot::Mutex`] for better performance than `std::sync::Mutex`
    /// - Requires the transaction to be safely shared between multiple futures
    ///
    /// # Example
    ///
    /// ```no_compile
    /// let results = repo.transaction_concurrent([
    ///     |tx_arc| async move {
    ///         let mut tx = tx_arc.lock();
    ///         repo.save_with_executor(&mut *tx, model1).await
    ///     },
    ///     |tx_arc| async move {
    ///         let mut tx = tx_arc.lock();
    ///         repo.save_with_executor(&mut *tx, model2).await
    ///     }
    /// ]).await;
    /// ```
    fn transaction_concurrent<'a, 'b, I, F, Fut, R, E>(
        &'a self,
        action_fns: I,
    ) -> impl Future<Output = Result<Vec<R>, E>> + Send + 'a
    where
        I: IntoIterator<Item = F> + Send + 'a,
        I::IntoIter: Send + 'a,
        F: FnOnce(Arc<parking_lot::Mutex<Transaction<'b, Database>>>) -> Fut + Send + 'a,
        Fut: Future<Output = Result<R, E>> + Send + 'a,
        R: Send + 'a,
        E: From<Error> + Send + 'a,
    {
        async move {
            let tx = self.pool().begin().await.map_err(E::from)?;
            let tx = Arc::new(parking_lot::Mutex::new(tx));

            // Create futures but don't await them yet
            let futures: Vec<_> = action_fns
                .into_iter()
                .map(|action_fn| action_fn(tx.clone()))
                .collect();

            // Execute all futures concurrently
            let results = try_join_all(futures).await;

            match results {
                Ok(values) => {
                    let tx = match Arc::into_inner(tx) {
                        Some(mutex) => mutex.into_inner(),
                        None => return Err(E::from(Error::PoolClosed)),
                    };

                    tx.commit().await.map_err(E::from)?;
                    Ok(values)
                }
                Err(e) => {
                    let tx = match Arc::into_inner(tx) {
                        Some(mutex) => mutex.into_inner(),
                        None => return Err(E::from(Error::PoolClosed)),
                    };

                    tx.rollback().await.map_err(E::from)?;
                    Err(e)
                }
            }
        }
    }

    /// Executes multiple operations and collects all results, committing only if all succeed.
    ///
    /// This method runs all actions sequentially, collecting results (both successes and failures).
    /// The transaction is committed only if all actions succeed; otherwise, it's rolled back.
    ///
    /// # Type Parameters
    ///
    /// * `I`: The iterator type
    /// * `F`: The action function type
    /// * `Fut`: The future type returned by each action
    /// * `R`: The result type
    /// * `E`: The error type, which must be convertible from [`Error`]
    ///
    /// # Parameters
    ///
    /// * `actions`: An iterator of functions that will be executed in the transaction
    ///
    /// # Returns
    ///
    /// A future that resolves to:
    /// * `Ok(Vec<R>)`: A vector of results if all operations succeeded
    /// * `Err(Vec<E>)`: A vector of all errors if any operation failed
    ///
    /// # Implementation Details
    ///
    /// 1. Begins a transaction from the repository's connection pool
    /// 2. Executes each action sequentially, collecting all results and errors
    /// 3. If any errors occurred, rolls back the transaction and returns all errors
    /// 4. If all operations succeeded, commits the transaction and returns the results
    ///
    /// # Example
    ///
    /// ```no_compile
    /// match repo.try_transaction([
    ///     |tx| async move { repo.save_with_executor(tx, model1).await },
    ///     |tx| async move { repo.save_with_executor(tx, model2).await }
    /// ]).await {
    ///     Ok(results) => println!("All operations succeeded"),
    ///     Err(errors) => println!("Some operations failed: {:?}", errors)
    /// }
    /// ```
    fn try_transaction<'a, I, F, Fut, R, E>(
        &'a self,
        actions: I,
    ) -> impl Future<Output = Result<Vec<R>, Vec<E>>> + Send + 'a
    where
        I: IntoIterator<Item = F> + Send + 'a,
        I::IntoIter: Send + 'a,
        F: FnOnce(&mut Transaction<'_, Database>) -> Fut + Send + 'a,
        Fut: Future<Output = Result<R, E>> + Send,
        R: Send + 'a,
        E: From<Error> + Send + 'a,
    {
        async move {
            let mut tx = self.pool().begin().await.map_err(|e| vec![E::from(e)])?;
            let mut results = Vec::new();
            let mut errors = Vec::new();

            for action in actions {
                match action(&mut tx).await {
                    Ok(result) => results.push(result),
                    Err(e) => errors.push(e),
                }
            }

            if errors.is_empty() {
                tx.commit().await.map_err(|e| vec![E::from(e)])?;
                Ok(results)
            } else {
                let _ = tx.rollback().await;
                Err(errors)
            }
        }
    }
}

impl<T, M> TransactionRepository<M> for T
where
    T: Repository<M>,
    M: Model,
{
}

/// Extension trait for Save operations with transactions.
///
/// This trait provides convenience methods for using transactions with repositories
/// that implement [`SaveRepository`]. It's automatically implemented for any type that
/// implements both [`SaveRepository<M>`] and [`TransactionRepository<M>`].
pub trait SaveRepositoryTransactionExt<M>: SaveRepository<M> + TransactionRepository<M>
where
    M: Model + Send + Sync,
{
    /// Saves a model in a transaction, ensuring atomicity.
    ///
    /// This method:
    /// 1. Creates a transaction using [`with_transaction`](TransactionRepository)
    /// 2. Calls [`save_with_executor`](SaveRepository::save_with_executor) with the transaction
    /// 3. Returns the model on successful save
    ///
    /// # Parameters
    ///
    /// * `model`: The model to save
    ///
    /// # Returns
    ///
    /// A future that resolves to:
    /// * `Ok(M)`: The saved model on success
    /// * `Err(crate::Error)`: The error if saving failed
    ///
    /// # Example
    ///
    /// ```no_compile
    /// let saved_model = repo.save_in_transaction(model).await?;
    /// ```
    fn save_in_transaction<'a>(
        &'a self,
        model: M,
    ) -> impl Future<Output = Result<M, crate::Error>> + Send + 'a
    where
        M: 'a,
    {
        self.with_transaction(move |mut tx| async move {
            let res = self.save_with_executor(&mut *tx, model).await;

            (res, tx)
        })
    }
}

// Blanket implementation for any repository that implements both required traits
impl<T, M> SaveRepositoryTransactionExt<M> for T
where
    T: SaveRepository<M> + TransactionRepository<M>,
    M: Model + Send + Sync,
{
}
