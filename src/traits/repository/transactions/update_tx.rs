use crate::prelude::*;
use std::future::Future;

pub trait UpdatableRepositoryTransaction<M: Model>:
    UpdatableRepository<M> + TransactionRepository<M>
{
    fn update_in_transaction<'a>(
        &'a self,
        model: M,
    ) -> impl Future<Output = Result<M, Error>> + Send + 'a
    where
        M: 'a,
    {
        self.with_transaction(move |mut tx| async move {
            let res = self.update_with_executor(&mut *tx, model).await;

            (res, tx)
        })
    }

    fn update_ref_in_transaction<'a, 'b>(
        &'a self,
        model: &'b M,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'a
    where
        'b: 'a,
        M: 'b,
    {
        self.with_transaction(move |mut tx| async move {
            let res = self.update_ref_with_executor(&mut *tx, model).await;

            (res, tx)
        })
    }
}
