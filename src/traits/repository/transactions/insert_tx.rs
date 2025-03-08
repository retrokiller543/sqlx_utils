use crate::prelude::*;
use std::future::Future;

pub trait InsertableRepositoryTransaction<M: Model>:
    InsertableRepository<M> + TransactionRepository<M>
{
    fn insert_in_transaction<'a>(
        &'a self,
        model: M,
    ) -> impl Future<Output = Result<M, Error>> + Send + 'a
    where
        M: 'a,
    {
        self.with_transaction(move |mut tx| async move {
            let res = self.insert_with_executor(&mut *tx, model).await;

            (res, tx)
        })
    }
}
