use crate::filter::InValues;
use crate::prelude::*;
use std::future::Future;

pub trait DeleteRepositoryTransaction<M: Model>:
    DeleteRepository<M> + TransactionRepository<M>
{
    fn delete_by_id_in_transaction<'a>(
        &'a self,
        id: impl Into<M::Id> + Send + 'a,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'a
    where
        M::Id: 'a,
    {
        self.with_transaction(move |mut tx| async move {
            let res = self.delete_by_id_with_executor(&mut *tx, id).await;

            (res, tx)
        })
    }

    fn delete_by_filter_in_transaction<'a>(
        &'a self,
        filter: impl SqlFilter<'a> + Send + 'a,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'a {
        self.with_transaction(move |mut tx| async move {
            let res = self.delete_by_filter_with_executor(&mut *tx, filter).await;

            (res, tx)
        })
    }

    fn delete_by_values_in_transaction<'a, I>(
        &'a self,
        column: &'static str,
        values: I,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'a
    where
        M::Id: ::sqlx::Type<Database> + ::sqlx::Encode<'a, Database> + 'a,
        I: IntoIterator<Item = M::Id> + Send + 'a,
        I::IntoIter: Send + 'a,
    {
        let filter = InValues::new(column.as_ref(), values);

        self.delete_by_filter_in_transaction(filter)
    }
}

impl<T, M> DeleteRepositoryTransaction<M> for T
where
    T: DeleteRepository<M> + TransactionRepository<M>,
    M: Model,
{
}
