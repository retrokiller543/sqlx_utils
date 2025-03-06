use std::fmt::Debug;
use crate::traits::{Model, Repository, SqlFilter};
use crate::types::Database;
use sqlx::{Database as DatabaseTrait, Executor, FromRow, QueryBuilder};
use std::future::Future;

pub trait FilterRepository<M>: Repository<M>
where
    M: Model + for<'r> FromRow<'r, <Database as DatabaseTrait>::Row> + Send + Unpin,
    Self: Sync,
{
    type Filter: for<'args> SqlFilter<'args, Database> + Debug + Send;

    fn query_builder<'args>() -> QueryBuilder<'args, Database>;

    #[inline(always)]
    #[tracing::instrument(skip(self, tx), level = "debug", parent = Self::repository_span(), name = "get_by_filter")]
    fn get_by_any_filter_raw<'a, F, E>(
        &'a self,
        tx: E,
        filter: F,
    ) -> impl Future<Output = crate::Result<Vec<M>>> + Send + 'a
    where
        F: for<'c> SqlFilter<'c, Database> + Debug + Send + 'a,
        E: for<'c> Executor<'c, Database = Database> + 'a,
    {
        async move {
            let mut query = Self::query_builder();
            filter.apply_filter(&mut query);
            query.build_query_as().fetch_all(tx).await.map_err(Into::into)
        }
    }

    #[inline(always)]
    async fn get_by_any_filter<'a, F>(
        &'a self,
        filter: F,
    ) -> crate::Result<Vec<M>>
    where
        F: for<'c> SqlFilter<'c, Database> + Debug + Send + 'a,
    {
            let pool = self.pool();
            self.get_by_any_filter_raw(pool, filter).await
    }

    #[inline(always)]
    async fn get_by_filter_raw<E>(
        &self,
        tx: E,
        filter: Self::Filter,
    ) -> crate::Result<Vec<M>>
    where
        E: for<'c> Executor<'c, Database = Database>,
    {
        self.get_by_any_filter_raw(tx, filter).await
    }

    #[inline(always)]
    async fn get_by_filter(
        &self,
        filter: Self::Filter,
    ) -> crate::Result<Vec<M>>
    {
        let pool = self.pool();
        self.get_by_any_filter_raw(pool, filter).await
    }
}
