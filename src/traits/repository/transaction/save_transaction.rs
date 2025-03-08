use crate::prelude::{Model, SaveRepository, TransactionRepository};
use std::future::Future;

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
